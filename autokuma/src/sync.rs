use crate::{
    config::Config,
    error::{Error, KumaError, Result},
    util::{group_by_prefix, ResultLogger},
};
use bollard::{
    container::ListContainersOptions, service::ContainerSummary, Docker, API_DEFAULT_VERSION,
};
use itertools::Itertools;
use kuma_client::{Client, Monitor, MonitorType, Tag, TagDefinition};
use log::{info, warn};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

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
        template_values: &Vec<(String, String)>,
    ) -> String {
        template_values
            .iter()
            .fold(config.into(), |config, (key, value)| {
                config.replace(&format!("{{{{{}}}}}", key), &value)
            })
    }

    fn get_defaults(&self, monitor_type: impl AsRef<str>) -> Vec<(String, String)> {
        vec![
            self.defaults.get("*"),
            self.defaults.get(monitor_type.as_ref()),
        ]
        .into_iter()
        .flat_map(|defaults| {
            defaults
                .into_iter()
                .map(|entry| entry.to_owned())
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
            .await?
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

    fn get_monitor_from_labels(
        &self,
        id: &str,
        monitor_type: &str,
        settings: Vec<(String, String)>,
        template_values: &Vec<(String, String)>,
    ) -> Result<Monitor> {
        let defaults = self.get_defaults(monitor_type);

        let config = Self::fill_templates(
            vec![("type".to_owned(), monitor_type.to_owned())]
                .into_iter()
                .chain(settings.into_iter())
                .chain(defaults.into_iter())
                .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
                .unique_by(|(key, _)| key.to_owned())
                .map(|(key, value)| format!("{} = {}", key, toml::Value::String(value)))
                .join("\n"),
            template_values,
        );

        let toml = toml::from_str::<serde_json::Value>(&config)
            .map_err(|e| Error::LabelParseError(e.to_string()))?;

        let monitor = serde_json::from_value::<Monitor>(toml)
            .log_warn(|e| format!("Error while parsing {}: {}!", id, e.to_string()))
            .map_err(|e| Error::LabelParseError(e.to_string()))?;

        monitor.validate(id)?;

        Ok(monitor)
    }

    fn get_monitors_from_labels(
        &self,
        labels: Vec<(String, String)>,
        template_values: &Vec<(String, String)>,
    ) -> Result<Vec<(String, Monitor)>> {
        group_by_prefix(
            labels.iter().map(|(key, value)| {
                (
                    key.trim_start_matches(&format!("{}.", self.config.docker.label_prefix)),
                    value,
                )
            }),
            ".",
        )
        .into_iter()
        .map(|(key, value)| (key, group_by_prefix(value, ".")))
        .flat_map(|(id, monitors)| {
            monitors.into_iter().map(move |(monitor_type, settings)| {
                self.get_monitor_from_labels(&id, &monitor_type, settings, template_values)
                    .map(|monitor| (id.clone(), monitor))
            })
        })
        .collect()
    }

    fn get_monitors_from_containers(
        &self,
        containers: &Vec<ContainerSummary>,
    ) -> Result<HashMap<String, Monitor>> {
        containers
            .into_iter()
            .map(|container| {
                let kuma_labels = container.labels.as_ref().map_or_else(
                    || vec![],
                    |labels| {
                        labels
                            .iter()
                            .filter(|(key, _)| {
                                key.starts_with(&format!("{}.", self.config.docker.label_prefix))
                            })
                            .map(|(key, value)| (key.to_owned(), value.to_owned()))
                            .collect::<Vec<_>>()
                    },
                );

                let template_values = [
                    ("container_id", &container.id),
                    ("image_id", &container.image_id),
                    ("image", &container.image),
                    (
                        "container_name",
                        &container
                            .names
                            .as_ref()
                            .and_then(|names| names.first().cloned()),
                    ),
                ]
                .into_iter()
                .filter_map(|(key, value)| {
                    value
                        .as_ref()
                        .map(|value| (key.to_owned(), value.to_owned()))
                })
                .collect_vec();

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
                    .tags
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

    async fn get_monitor_from_file(file: impl AsRef<str>) -> Result<Option<(String, Monitor)>> {
        let file = file.as_ref();
        let id = std::path::Path::new(file)
            .file_stem()
            .and_then(|os| os.to_str().map(|str| str.to_owned()))
            .ok_or_else(|| Error::IO(format!("Unable to determine file: '{}'", file)))?;

        Ok(if file.ends_with(".json") {
            let content = tokio::fs::read_to_string(file)
                .await
                .map_err(|e| Error::IO(e.to_string()))?;
            Some((
                id,
                serde_json::from_str(&content)
                    .map_err(|e| Error::DeserializeError(e.to_string()))?,
            ))
        } else if file.ends_with(".toml") {
            let content = tokio::fs::read_to_string(file)
                .await
                .map_err(|e| Error::IO(e.to_string()))?;
            Some((
                id,
                toml::from_str(&content).map_err(|e| Error::DeserializeError(e.to_string()))?,
            ))
        } else {
            None
        })
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
            .tags
            .iter()
            .filter_map(|tag| tag.tag_id.as_ref().map(|id| (*id, tag)))
            .collect::<HashMap<_, _>>();

        let merged_tags: Vec<Tag> = new
            .common_mut()
            .tags
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

        new.common_mut().tags = merged_tags;

        serde_merge::omerge(current, new).unwrap()
    }

    async fn do_sync(&self) -> Result<()> {
        let kuma = Client::connect(self.config.kuma.clone()).await?;

        let autokuma_tag = self.get_autokuma_tag(&kuma).await?;
        let current_monitors = self.get_managed_monitors(&kuma).await?;

        let mut new_monitors: HashMap<String, Monitor> = HashMap::new();

        if self.config.docker.enabled {
            let docker = Docker::connect_with_socket(
                &self.config.docker.socket_path,
                120,
                API_DEFAULT_VERSION,
            )?;

            let containers = self.get_kuma_containers(&docker).await?;
            new_monitors.extend(self.get_monitors_from_containers(&containers)?);
        }

        if tokio::fs::metadata(&self.config.static_monitors)
            .await
            .is_ok_and(|md| md.is_dir())
        {
            let mut dir = tokio::fs::read_dir(&self.config.static_monitors)
                .await
                .log_warn(|e| e.to_string());

            if let Ok(dir) = &mut dir {
                loop {
                    if let Some(f) = dir
                        .next_entry()
                        .await
                        .map_err(|e| Error::IO(e.to_string()))?
                    {
                        if let Some((id, monitor)) =
                            Self::get_monitor_from_file(f.path().to_string_lossy()).await?
                        {
                            new_monitors.insert(id, monitor);
                        }
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
                    .id
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
            monitor.common_mut().tags.push(tag);

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
                || merge.common().parent_name.is_some() != current.common().parent.is_some()
                || merge
                    .common()
                    .parent_name
                    .as_ref()
                    .is_some_and(|name| Some(name) != current.common().parent.map(|id| groups[&id]))
            {
                info!("Updating monitor: {}", id);
                kuma.edit_monitor(merge).await?;
            }
        }

        for (id, monitor) in to_delete {
            info!("Deleting monitor: {}", id);
            if let Some(id) = monitor.common().id {
                kuma.delete_monitor(id).await?;
            }
        }

        kuma.disconnect().await?;

        Ok(())
    }

    pub async fn run(&self) {
        loop {
            if let Err(err) = self.do_sync().await {
                warn!("Encountered error during sync: {}", err);
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
