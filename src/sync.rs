use crate::{
    config::Config,
    kuma::{Client, Monitor, Tag},
    util::{group_by_prefix, ResultLogger},
};
use bollard::{
    container::ListContainersOptions, service::ContainerSummary, Docker, API_DEFAULT_VERSION,
};
use itertools::Itertools;
use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct Sync {
    config: Arc<Config>,
}

impl Sync {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config: config }
    }

    async fn get_kuma_containers(&self, docker: &Docker) -> Vec<ContainerSummary> {
        docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .unwrap()
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
            .collect::<Vec<_>>()
    }

    fn get_monitor_from_labels(
        &self,
        id: &str,
        monitor_type: &str,
        settings: Vec<(String, String)>,
    ) -> Option<Monitor> {
        let config = vec![("type".to_owned(), monitor_type.to_owned())]
            .into_iter()
            .chain(settings.into_iter())
            .map(|(key, value)| format!("{} = {}", key, toml::Value::String(value)))
            .join("\n");

        if let Ok(toml) = toml::from_str::<serde_json::Value>(&config) {
            return serde_json::from_value::<Monitor>(toml)
                .on_error_log(|e| format!("Error while parsing {}: {}!", id, e.to_string()))
                .ok();
        }

        None
    }

    fn get_monitors_from_labels(&self, labels: Vec<(String, String)>) -> Vec<(String, Monitor)> {
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
            monitors
                .into_iter()
                .filter_map(move |(monitor_type, settings)| {
                    self.get_monitor_from_labels(&id, &monitor_type, settings)
                        .map(|monitor| (id.clone(), monitor))
                })
        })
        .collect()
    }

    fn get_monitors_from_containers(
        &self,
        containers: &Vec<ContainerSummary>,
    ) -> HashMap<String, Monitor> {
        containers
            .into_iter()
            .flat_map(|container| {
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

                self.get_monitors_from_labels(kuma_labels)
            })
            .sorted_by(|a, b| {
                Ord::cmp(
                    &if a.1.common().parent.is_some() { 1 } else { -1 },
                    &if b.1.common().parent.is_some() { 1 } else { -1 },
                )
            })
            .collect::<HashMap<_, _>>()
    }

    async fn get_autokuma_tag(&self, kuma: &Client) -> Tag {
        match kuma
            .tags()
            .await
            .into_iter()
            .find(|tag| tag.name.as_deref() == Some(&self.config.kuma.tag_name))
        {
            Some(tag) => tag,
            None => {
                kuma.add_tag(Tag {
                    name: Some(self.config.kuma.tag_name.clone()),
                    color: Some(self.config.kuma.tag_color.clone()),
                    ..Default::default()
                })
                .await
            }
        }
    }

    async fn get_managed_monitors(&self, kuma: &Client) -> HashMap<String, Monitor> {
        kuma.monitors()
            .await
            .into_iter()
            .filter_map(|(_, monitor)| {
                match monitor
                    .common()
                    .tags
                    .iter()
                    .filter(|tag| tag.name.as_deref() == Some(&self.config.kuma.tag_name))
                    .find_map(|tag| tag.value.as_ref())
                {
                    Some(id) => Some((id.to_owned(), monitor)),
                    None => None,
                }
            })
            .collect::<HashMap<_, _>>()
    }

    pub async fn run(&self) {
        let docker =
            Docker::connect_with_socket(&self.config.docker.socket_path, 120, API_DEFAULT_VERSION)
                .unwrap();
        let kuma = Client::connect(self.config.clone()).await;

        loop {
            let autokuma_tag = self.get_autokuma_tag(&kuma).await;
            let containers = self.get_kuma_containers(&docker).await;
            let new_monitors = self.get_monitors_from_containers(&containers);
            let current_monitors = self.get_managed_monitors(&kuma).await;

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
                println!("Creating new monitor: {}", id);

                let mut monitor = monitor.clone();
                let mut tag = autokuma_tag.clone();

                tag.value = Some(id.clone());
                monitor.common_mut().tags.push(tag);

                kuma.add_monitor(monitor).await;
            }

            for (id, current, new) in to_update {
                let merge: Monitor = serde_merge::omerge(current, new).unwrap();

                if current != &merge {
                    println!("Updating monitor: {}", id);
                    kuma.edit_monitor(merge).await;
                }
            }

            for (id, monitor) in to_delete {
                println!("Deleting monitor: {}", id);
                if let Some(id) = monitor.common().id {
                    kuma.delete_monitor(id).await;
                }
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
