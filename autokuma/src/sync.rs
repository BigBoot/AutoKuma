use crate::{
    config::{Config, DeleteBehavior, DockerSource},
    error::{Error, KumaError, Result},
    util::{group_by_prefix, FlattenValue as _, ResultLogger},
};
use bollard::{
    container::ListContainersOptions,
    service::{ContainerSummary, ListServicesOptions, Service},
    Docker,
};
use itertools::Itertools;
use kuma_client::{
    monitor::{Monitor, MonitorType},
    tag::{Tag, TagDefinition},
    Client,
};
use log::{debug, info, warn};
use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
    env,
    error::Error as _,
    sync::Arc,
    time::Duration,
};
use tera::Tera;
use unescaper::unescape;

pub struct Sync {
    config: Arc<Config>,
    defaults: BTreeMap<String, Vec<(String, String)>>,
}

impl Sync {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let defaults = config
            .default_settings
            .lines()
            .map(|line| {
                line.split_once(":")
                    .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                    .ok_or_else(|| {
                        Error::InvalidConfig("kuma.default_settings".to_owned(), line.to_owned())
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            config: config.clone(),
            defaults: group_by_prefix(defaults, "."),
        })
    }

    fn fill_templates(
        config: impl Into<String>,
        template_values: &tera::Context,
    ) -> Result<String> {
        let config = config.into();
        let mut tera = Tera::default();

        tera.add_raw_template(&config, &config)
            .and_then(|_| tera.render(&config, template_values))
            .map_err(|e| {
                Error::LabelParseError(format!(
                    "{}\nContext: {:?}",
                    e.source().unwrap(),
                    &template_values.get("container")
                ))
            })
    }

    fn get_defaults(&self, monitor_type: impl AsRef<str>) -> Vec<(String, serde_json::Value)> {
        vec![
            self.defaults.get("*"),
            self.defaults.get(monitor_type.as_ref()),
        ]
        .into_iter()
        .flat_map(|defaults| {
            defaults
                .into_iter()
                .map(|entry| {
                    entry
                        .into_iter()
                        .map(|(key, value)| (key.to_owned(), json!(value)))
                })
                .collect_vec()
        })
        .flatten()
        .collect_vec()
    }

    async fn get_kuma_containers(&self, docker: &Docker) -> Result<Vec<ContainerSummary>> {
        Ok(docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .log_warn(std::module_path!(), |_| {
                format!(
                    "Using DOCKER_HOST={}",
                    env::var("DOCKER_HOST").unwrap_or_else(|_| "None".to_owned())
                )
            })?
            .into_iter()
            .filter(|c| {
                c.labels.as_ref().map_or_else(
                    || false,
                    |labels| {
                        labels.keys().any(|key| {
                            key.starts_with(&format!("{}.", self.config.docker.label_prefix))
                        })
                    },
                )
            })
            .collect::<Vec<_>>())
    }

    async fn get_kuma_services(&self, docker: &Docker) -> Result<Vec<Service>> {
        Ok(docker
            .list_services(Some(ListServicesOptions::<String> {
                ..Default::default()
            }))
            .await
            .log_warn(std::module_path!(), |_| {
                format!(
                    "Using DOCKER_HOST={}",
                    env::var("DOCKER_HOST").unwrap_or_else(|_| "None".to_owned())
                )
            })?
            .into_iter()
            .filter(|c| {
                c.spec.as_ref().map_or_else(
                    || false,
                    |spec| {
                        spec.labels.as_ref().map_or_else(
                            || false,
                            |labels| {
                                labels.keys().any(|key| {
                                    key.starts_with(&format!(
                                        "{}.",
                                        self.config.docker.label_prefix
                                    ))
                                })
                            },
                        )
                    },
                )
            })
            .collect::<Vec<_>>())
    }

    fn get_monitor_from_settings(
        &self,
        id: &str,
        monitor_type: &str,
        settings: Vec<(String, serde_json::Value)>,
        template_values: &tera::Context,
    ) -> Result<Monitor> {
        let defaults = self.get_defaults(monitor_type);

        let config = Self::fill_templates(
            vec![("type".to_owned(), json!(monitor_type.to_owned()))]
                .into_iter()
                .chain(settings.into_iter())
                .chain(defaults.into_iter())
                .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
                .unique_by(|(key, _)| key.to_owned())
                .map(|(key, value)| format!("{} = {}", key, value))
                .join("\n"),
            template_values,
        )?;

        let toml = toml::from_str::<serde_json::Value>(&config)
            .map_err(|e| Error::LabelParseError(e.to_string()))?;

        let monitor = serde_json::from_value::<Monitor>(toml)
            .log_warn(std::module_path!(), |e| {
                format!("Error while parsing {}: {}!", id, e.to_string())
            })
            .map_err(|e| Error::LabelParseError(e.to_string()))?;

        monitor.validate(id)?;

        Ok(monitor)
    }

    fn get_monitors_from_labels(
        &self,
        labels: Vec<(String, String)>,
        template_values: &tera::Context,
    ) -> Result<Vec<(String, Monitor)>> {
        let entries = labels
            .iter()
            .flat_map(|(key, value)| {
                if key.starts_with("__") {
                    let snippet = self
                        .config
                        .snippets
                        .get(key.trim_start_matches("__"))
                        .log_warn(std::module_path!(), || {
                            format!("Snippet '{}' not found!", key)
                        });

                    let args =
                        serde_json::from_str::<Vec<serde_json::Value>>(&format!("[{}]", value))
                            .log_warn(std::module_path!(), |e| {
                                format!("Error while parsing snippet arguments: {}", e.to_string())
                            })
                            .ok();

                    if let (Some(snippet), Some(args)) = (snippet, args) {
                        let mut template_values = template_values.clone();
                        template_values.insert("args", &args);

                        if let Ok(snippet) = Self::fill_templates(snippet, &template_values)
                            .log_warn(std::module_path!(), |e| {
                                format!("Error while parsing snippet: {}", e.to_string())
                            })
                        {
                            snippet
                                .lines()
                                .filter(|line| !line.trim().is_empty())
                                .flat_map(|line| {
                                    line.split_once(": ")
                                        .map(|(key, value)| {
                                            (
                                                key.trim_start().to_owned(),
                                                unescape(value)
                                                    .unwrap_or_else(|_| value.to_owned()),
                                            )
                                        })
                                        .log_warn(std::module_path!(), || {
                                            format!("Invalid snippet line: '{}'", line)
                                        })
                                })
                                .collect::<Vec<_>>()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                } else {
                    vec![(key.to_owned(), value.to_owned())]
                }
            })
            .collect::<Vec<_>>();

        group_by_prefix(entries, ".")
            .into_iter()
            .map(|(key, value)| (key, group_by_prefix(value, ".")))
            .flat_map(|(id, monitors)| {
                monitors.into_iter().map(move |(monitor_type, settings)| {
                    self.get_monitor_from_settings(
                        &id,
                        &monitor_type,
                        settings
                            .into_iter()
                            .map(|(key, value)| (key, json!(value)))
                            .collect_vec(),
                        template_values,
                    )
                    .map(|monitor| (id.clone(), monitor))
                })
            })
            .collect()
    }

    fn get_kuma_labels(
        &self,
        labels: Option<&HashMap<String, String>>,
        template_values: &tera::Context,
    ) -> Result<Vec<(String, String)>> {
        labels.as_ref().map_or_else(
            || Ok(vec![]),
            |labels| {
                labels
                    .iter()
                    .filter(|(key, _)| {
                        key.starts_with(&format!("{}.", self.config.docker.label_prefix))
                    })
                    .map(|(key, value)| {
                        Self::fill_templates(
                            key.trim_start_matches(&format!(
                                "{}.",
                                self.config.docker.label_prefix
                            )),
                            &template_values,
                        )
                        .map(|key| (key, value.to_owned()))
                    })
                    .collect::<Result<Vec<_>>>()
            },
        )
    }

    fn get_monitors_from_containers(
        &self,
        containers: &Vec<ContainerSummary>,
    ) -> Result<HashMap<String, Monitor>> {
        containers
            .into_iter()
            .map(|container| {
                let mut template_values = tera::Context::new();
                template_values.insert("container_id", &container.id);
                template_values.insert("image_id", &container.image_id);
                template_values.insert("image", &container.image);
                template_values.insert(
                    "container_name",
                    &container.names.as_ref().and_then(|names| {
                        names.first().map(|s| s.trim_start_matches("/").to_owned())
                    }),
                );

                template_values.insert("container", &container);

                let kuma_labels =
                    self.get_kuma_labels(container.labels.as_ref(), &template_values)?;

                self.get_monitors_from_labels(kuma_labels, &template_values)
            })
            .flatten_ok()
            .try_collect()
    }

    fn get_monitors_from_services(
        &self,
        services: &Vec<Service>,
    ) -> Result<HashMap<String, Monitor>> {
        services
            .into_iter()
            .map(|service| {
                let mut template_values = tera::Context::new();

                template_values.insert("service", &service);

                let spec = service.spec.as_ref();
                let labels = spec.and_then(|spec| spec.labels.as_ref());

                let kuma_labels = self.get_kuma_labels(labels, &template_values)?;

                self.get_monitors_from_labels(kuma_labels, &template_values)
            })
            .flatten_ok()
            .try_collect()
    }

    async fn get_autokuma_tag(&self, kuma: &Client) -> Result<TagDefinition> {
        match kuma
            .get_tags()
            .await?
            .into_iter()
            .find(|tag| tag.name.as_deref() == Some(&self.config.tag_name))
        {
            Some(tag) => Ok(tag),
            None => Ok(kuma
                .add_tag(TagDefinition {
                    name: Some(self.config.tag_name.clone()),
                    color: Some(self.config.tag_color.clone()),
                    ..Default::default()
                })
                .await?),
        }
    }

    async fn get_managed_monitors(&self, kuma: &Client) -> Result<HashMap<String, Monitor>> {
        Ok(kuma
            .get_monitors()
            .await?
            .into_iter()
            .filter_map(|(_, monitor)| {
                match monitor
                    .common()
                    .tags()
                    .iter()
                    .filter(|tag| tag.name.as_deref() == Some(&self.config.tag_name))
                    .find_map(|tag| tag.value.as_ref())
                {
                    Some(id) => Some((id.to_owned(), monitor)),
                    None => None,
                }
            })
            .collect::<HashMap<_, _>>())
    }

    async fn get_monitor_from_file(&self, file: impl AsRef<str>) -> Result<(String, Monitor)> {
        let file = file.as_ref();
        let id = std::path::Path::new(file)
            .file_stem()
            .and_then(|os| os.to_str().map(|str| str.to_owned()))
            .ok_or_else(|| Error::IO(format!("Unable to determine file: '{}'", file)))?;

        let value: Option<serde_json::Value> = if file.ends_with(".json") {
            let content: String = tokio::fs::read_to_string(file)
                .await
                .map_err(|e| Error::IO(e.to_string()))?;

            Some(
                serde_json::from_str(&content)
                    .map_err(|e| Error::DeserializeError(e.to_string()))?,
            )
        } else if file.ends_with(".toml") {
            let content = tokio::fs::read_to_string(file)
                .await
                .map_err(|e| Error::IO(e.to_string()))?;

            Some(toml::from_str(&content).map_err(|e| Error::DeserializeError(e.to_string()))?)
        } else {
            None
        };

        let values = value
            .ok_or_else(|| {
                Error::DeserializeError(format!(
                    "Unsupported static monitor file type: {}, supported: .json, .toml",
                    file
                ))
            })
            .and_then(|v| v.flatten())?;

        let monitor_type = values
            .iter()
            .find(|(key, _)| key == "type")
            .and_then(|(_, value)| value.as_str().map(|s| s.to_owned()))
            .ok_or_else(|| {
                Error::DeserializeError(format!("Static monitor {} is missing `type`", file))
            })?;

        let context = tera::Context::new();
        let monitor = self.get_monitor_from_settings(&id, &monitor_type, values, &context)?;

        Ok((id, monitor))
    }

    fn merge_monitors(
        &self,
        current: &Monitor,
        new: &Monitor,
        addition_tags: Option<Vec<Tag>>,
    ) -> Monitor {
        let mut new = new.clone();

        let current_tags = current
            .common()
            .tags()
            .iter()
            .filter_map(|tag| tag.tag_id.as_ref().map(|id| (*id, tag)))
            .collect::<HashMap<_, _>>();

        let merged_tags: Vec<Tag> = new
            .common_mut()
            .tags_mut()
            .drain(..)
            .chain(addition_tags.unwrap_or_default())
            .map(|new_tag| {
                new_tag
                    .tag_id
                    .as_ref()
                    .and_then(|id| {
                        current_tags.get(id).and_then(|current_tag| {
                            serde_merge::omerge(current_tag, &new_tag).unwrap()
                        })
                    })
                    .unwrap_or_else(|| new_tag)
            })
            .collect_vec();

        *new.common_mut().tags_mut() = merged_tags;

        serde_merge::omerge(current, new).unwrap()
    }

    async fn do_sync(&self) -> Result<()> {
        let kuma =
            Client::connect_with_tag_name(self.config.kuma.clone(), self.config.tag_name.clone())
                .await?;

        let autokuma_tag = self.get_autokuma_tag(&kuma).await?;
        let current_monitors = self.get_managed_monitors(&kuma).await?;

        let mut new_monitors: HashMap<String, Monitor> = HashMap::new();

        if self.config.docker.enabled {
            let docker_hosts = self
                .config
                .docker
                .hosts
                .clone()
                .map(|f| f.into_iter().map(Some).collect::<Vec<_>>())
                .unwrap_or_else(|| {
                    vec![self
                        .config
                        .docker
                        .socket_path
                        .as_ref()
                        .and_then(|path| Some(format!("unix://{}", path)))]
                });

            for docker_host in docker_hosts {
                if let Some(docker_host) = &docker_host {
                    env::set_var("DOCKER_HOST", docker_host);
                }

                let docker =
                    Docker::connect_with_defaults().log_warn(std::module_path!(), |_| {
                        format!(
                            "Using DOCKER_HOST={}",
                            env::var("DOCKER_HOST").unwrap_or_else(|_| "None".to_owned())
                        )
                    })?;

                if self.config.docker.source == DockerSource::Containers
                    || self.config.docker.source == DockerSource::Both
                {
                    let containers = self.get_kuma_containers(&docker).await?;
                    new_monitors.extend(self.get_monitors_from_containers(&containers)?);
                }

                if self.config.docker.source == DockerSource::Services
                    || self.config.docker.source == DockerSource::Both
                {
                    let services = self.get_kuma_services(&docker).await?;
                    new_monitors.extend(self.get_monitors_from_services(&services)?);
                }

                let containers = self.get_kuma_containers(&docker).await?;
                new_monitors.extend(self.get_monitors_from_containers(&containers)?);
            }
        }

        let static_monitor_path = self
            .config
            .static_monitors
            .clone()
            .unwrap_or_else(|| {
                dirs::config_local_dir()
                    .map(|dir| {
                        dir.join("autokuma")
                            .join("static-monitors")
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_default()
            })
            .to_owned();

        if tokio::fs::metadata(&static_monitor_path)
            .await
            .is_ok_and(|md| md.is_dir())
        {
            let mut dir = tokio::fs::read_dir(&static_monitor_path)
                .await
                .log_warn(std::module_path!(), |e| e.to_string());

            if let Ok(dir) = &mut dir {
                loop {
                    if let Some(f) = dir
                        .next_entry()
                        .await
                        .map_err(|e| Error::IO(e.to_string()))?
                    {
                        let (id, monitor) = self
                            .get_monitor_from_file(f.path().to_string_lossy())
                            .await?;
                        new_monitors.insert(id, monitor);
                    } else {
                        break;
                    }
                }
            }
        }

        let groups = current_monitors
            .iter()
            .filter(|(_, monitor)| monitor.monitor_type() == MonitorType::Group)
            .filter_map(|(id, monitor)| {
                monitor
                    .common()
                    .id()
                    .as_ref()
                    .map(|parent_id| (parent_id, id))
            })
            .collect::<HashMap<_, _>>();

        let to_delete = current_monitors
            .iter()
            .filter(|(id, _)| !new_monitors.contains_key(*id))
            .collect_vec();

        let to_create = new_monitors
            .iter()
            .filter(|(id, _)| !current_monitors.contains_key(*id))
            .collect_vec();

        let to_update = current_monitors
            .keys()
            .filter_map(
                |id| match (current_monitors.get(id), new_monitors.get(id)) {
                    (Some(current), Some(new)) => Some((id, current, new)),
                    _ => None,
                },
            )
            .collect_vec();

        for (id, monitor) in to_create {
            info!("Creating new monitor: {}", id);

            let mut monitor = monitor.clone();
            let mut tag = Tag::from(autokuma_tag.clone());

            tag.value = Some(id.clone());
            monitor.common_mut().tags_mut().push(tag);

            match kuma.add_monitor(monitor).await {
                Ok(_) => Ok(()),
                Err(KumaError::GroupNotFound(group)) => {
                    warn!(
                        "Cannot create monitor {} because group {} does not exist",
                        id, group
                    );
                    Ok(())
                }
                Err(err) => Err(err),
            }?;
        }

        for (id, current, new) in to_update {
            let mut tag = Tag::from(autokuma_tag.clone());
            tag.value = Some(id.clone());

            let merge: Monitor = self.merge_monitors(current, new, Some(vec![tag]));

            if current != &merge
                || merge.common().parent_name().as_ref() != Some(id)
                    && (merge.common().parent_name().is_some()
                        != current.common().parent().is_some()
                        || merge.common().parent_name().as_ref().is_some_and(|name| {
                            Some(name) != current.common().parent().map(|id| groups[&id])
                        }))
            {
                info!("Updating monitor: {}", id);
                debug!(
                    "\n======= OLD =======\n{}\n===================\n\n======= NEW =======\n{}\n===================", 
                    serde_json::to_string_pretty(&current).unwrap(),
                    serde_json::to_string_pretty(&merge).unwrap()
                );
                kuma.edit_monitor(merge).await?;
            }
        }

        if self.config.on_delete == DeleteBehavior::Delete {
            for (id, monitor) in to_delete {
                info!("Deleting monitor: {}", id);
                if let Some(id) = monitor.common().id() {
                    kuma.delete_monitor(*id).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn run(&self) {
        loop {
            if let Err(err) = self.do_sync().await {
                warn!("Encountered error during sync: {}", err);
            }
            tokio::time::sleep(Duration::from_secs_f64(self.config.sync_interval)).await;
        }
    }
}
