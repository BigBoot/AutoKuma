use crate::{
    app_state::AppState,
    error::{Error, Result},
    name::Name,
    util::{fill_templates, group_by_prefix, FlattenValue},
};
use itertools::Itertools;
use kuma_client::{
    docker_host::DockerHost,
    monitor::{Monitor, MonitorType},
    notification::Notification,
    status_page::StatusPage,
    tag::{Tag, TagDefinition},
    util::ResultLogger,
};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use strum::Display;
use unescaper::unescape;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "EntityWrapper", into = "EntityWrapper")]
pub enum Entity {
    DockerHost(DockerHost),
    Notification(Notification),
    Monitor(Monitor),
    Tag(TagDefinition),
    StatusPage(StatusPage),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
pub enum EntityType {
    DockerHost,
    Notification,
    Monitor(MonitorType),
    Tag,
    StatusPage,
}

impl Entity {
    pub fn entity_type(&self) -> EntityType {
        match self {
            Entity::DockerHost(_) => EntityType::DockerHost,
            Entity::Notification(_) => EntityType::Notification,
            Entity::Monitor(monitor) => EntityType::Monitor(monitor.monitor_type()),
            Entity::Tag(_) => EntityType::Tag,
            Entity::StatusPage(_) => EntityType::StatusPage,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum EntityWrapper {
    DockerHost {
        #[serde(flatten)]
        docker_host: DockerHostTagged,
    },
    Notification {
        #[serde(flatten)]
        notification: NotificationTagged,
    },
    Tag {
        #[serde(flatten)]
        tag: TagTagged,
    },
    Monitor {
        #[serde(flatten)]
        monitor: Monitor,
    },
    StatusPage {
        #[serde(flatten)]
        status_page: StatusPageTagged,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum NotificationTagged {
    #[serde(rename = "notification")]
    Notification {
        #[serde(flatten)]
        notification: Notification,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum DockerHostTagged {
    #[serde(rename = "docker_host")]
    DockerHost {
        #[serde(flatten)]
        docker_host: DockerHost,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TagTagged {
    #[serde(rename = "tag")]
    Tag {
        #[serde(flatten)]
        tag: TagDefinition,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum StatusPageTagged {
    #[serde(rename = "status_page")]
    StatusPage {
        #[serde(flatten)]
        status_page: StatusPage,
    },
}

impl From<EntityWrapper> for Entity {
    fn from(wrapper: EntityWrapper) -> Self {
        match wrapper {
            EntityWrapper::DockerHost {
                docker_host: DockerHostTagged::DockerHost { docker_host },
            } => Entity::DockerHost(docker_host),

            EntityWrapper::Notification {
                notification: NotificationTagged::Notification { notification },
            } => Entity::Notification(notification),

            EntityWrapper::Tag {
                tag: TagTagged::Tag { tag },
            } => Entity::Tag(tag),

            EntityWrapper::StatusPage {
                status_page: StatusPageTagged::StatusPage { status_page },
            } => Entity::StatusPage(status_page),

            EntityWrapper::Monitor { monitor } => Entity::Monitor(monitor),
        }
    }
}

impl From<Entity> for EntityWrapper {
    fn from(entity: Entity) -> Self {
        match entity {
            Entity::DockerHost(docker_host) => EntityWrapper::DockerHost {
                docker_host: DockerHostTagged::DockerHost { docker_host },
            },

            Entity::Notification(notification) => EntityWrapper::Notification {
                notification: NotificationTagged::Notification { notification },
            },

            Entity::Tag(tag) => EntityWrapper::Tag {
                tag: TagTagged::Tag { tag },
            },

            Entity::StatusPage(status_page) => EntityWrapper::StatusPage {
                status_page: StatusPageTagged::StatusPage { status_page },
            },

            Entity::Monitor(monitor) => EntityWrapper::Monitor { monitor },
        }
    }
}

pub fn get_entities_from_labels(
    state: Arc<AppState>,
    labels: Vec<(String, String)>,
    template_values: &tera::Context,
) -> Result<Vec<(String, Entity)>> {
    let entries = labels
        .iter()
        .flat_map(|(key, value)| {
            if key.starts_with("__") {
                let snippet = state
                    .config
                    .snippets
                    .get(key.trim_start_matches("__"))
                    .log_warn(std::module_path!(), || {
                        format!("Snippet '{}' not found!", key)
                    });

                let args = serde_json::from_str::<Vec<serde_json::Value>>(&format!("[{}]", value))
                    .log_warn(std::module_path!(), |e| {
                        format!("Error while parsing snippet arguments: {}", e.to_string())
                    })
                    .ok();

                if let (Some(snippet), Some(args)) = (snippet, args) {
                    let mut template_values = template_values.clone();
                    template_values.insert("args", &args);

                    if let Ok(snippet) =
                        fill_templates(state.config.clone(), snippet, &template_values)
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
                                            unescape(value).unwrap_or_else(|_| value.to_owned()),
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
        .flat_map(|(id, entities)| {
            let state = state.clone();
            entities
                .into_iter()
                .filter_map(move |(entity_type, settings)| {
                    let result = get_entity_from_settings(
                        state.clone(),
                        &id,
                        &entity_type,
                        settings
                            .into_iter()
                            .map(|(key, value)| (key, json!(value)))
                            .collect_vec(),
                        template_values,
                    )
                    .map(|entity| (id.clone(), entity));

                    match result {
                        Err(Error::NameNotFound(name)) => {
                            warn!(
                                "Cannot create monitor {} because referenced {} with {} is not found",
                                id,
                                name.type_name(),
                                name.name()
                            );
                            None
                        }
                        result => Some(result),
                    }
                })
        })
        .collect()
}

fn resolve_names(state: Arc<AppState>, monitor: &mut Monitor) -> Result<()> {
    if let Some(group_name) = monitor.common().parent_name().clone() {
        let name = Name::Monitor(group_name.clone());
        let group_id = state
            .db
            .get_id(name.clone())
            .ok()
            .flatten()
            .ok_or_else(|| Error::NameNotFound(name))?;

        *monitor.common_mut().parent_mut() = Some(group_id);
    }

    if let Some(notification_names) = monitor.common().notification_names() {
        let notification_id_list = notification_names
            .iter()
            .map(|notification_name| {
                let name = Name::Notification(notification_name.clone());
                let id = state
                    .db
                    .get_id::<i32>(name.clone())
                    .ok()
                    .flatten()
                    .ok_or_else(|| Error::NameNotFound(name))?;

                Ok((id.to_string(), true))
            })
            .collect::<Result<HashMap<String, bool>>>()?;

        monitor
            .common_mut()
            .notification_id_list_mut()
            .get_or_insert(HashMap::new())
            .extend(notification_id_list.into_iter());
    }

    if let Some(tag_names) = monitor.common().tag_names() {
        let mut tags = tag_names
            .iter()
            .map(|tag_value| {
                let name = Name::Tag(tag_value.name.clone());
                let id = state
                    .db
                    .get_id(name.clone())
                    .ok()
                    .flatten()
                    .ok_or_else(|| Error::NameNotFound(name))?;

                Ok(Tag {
                    tag_id: Some(id),
                    name: None,
                    value: tag_value.value.clone(),
                    ..Default::default()
                })
            })
            .collect::<Result<Vec<Tag>>>()?;

        monitor.common_mut().tags_mut().append(&mut tags);
    }

    match monitor {
        Monitor::Docker {
            value: docker_monitor,
        } => {
            if let Some(docker_host_name) = &docker_monitor.docker_host_name {
                let name = Name::DockerHost(docker_host_name.clone());
                let docker_host_id = state
                    .db
                    .get_id(name.clone())
                    .ok()
                    .flatten()
                    .ok_or_else(|| Error::NameNotFound(name))?;

                docker_monitor.docker_host = Some(docker_host_id);
            }
        }
        _ => {}
    }

    return Ok(());
}

pub fn get_entity_from_value(
    state: Arc<AppState>,
    id: String,
    value: serde_json::Value,
    context: tera::Context,
) -> Result<Entity> {
    let values = value.flatten()?;

    let entity_type = values
        .iter()
        .find(|(key, _)| key == "type")
        .and_then(|(_, value)| value.as_str().map(|s| s.to_owned()))
        .ok_or_else(|| Error::DeserializeError(format!("Monitor {} is missing `type`", id)))?;

    let entity = get_entity_from_settings(state, &id, &entity_type, values, &context)?;

    Ok(entity)
}

pub fn get_entity_from_settings(
    state: Arc<AppState>,
    id: &str,
    entity_type: &str,
    settings: Vec<(String, serde_json::Value)>,
    context: &tera::Context,
) -> Result<Entity> {
    let defaults = state.get_defaults(entity_type);

    let config = fill_templates(
        state.config.clone(),
        vec![("type".to_owned(), json!(entity_type.to_owned()))]
            .into_iter()
            .chain(settings.into_iter())
            .chain(defaults.into_iter())
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
            .unique_by(|(key, _)| key.to_owned())
            .map(|(key, value)| format!("{} = {}", key, value))
            .join("\n"),
        context,
    )?;

    let toml = toml::from_str::<serde_json::Value>(&config)
        .map_err(|e| Error::LabelParseError(e.to_string()))?;

    let mut entity = serde_json::from_value::<Entity>(toml)
        .log_warn(std::module_path!(), |e| {
            format!("Error while parsing {}: {}!", id, e.to_string())
        })
        .map_err(|e| Error::LabelParseError(e.to_string()))?;

    if let Entity::Monitor(monitor) = &mut entity {
        monitor.validate(id)?;
        resolve_names(state, monitor)?;
    }

    Ok(entity)
}

pub fn merge_entities(current: &Entity, new: &Entity, addition_tags: Option<Vec<Tag>>) -> Entity {
    let mut new = new.clone();

    if let (Entity::Monitor(new_monitor), Entity::Monitor(current_monitor)) = (&mut new, &current) {
        let current_tags = current_monitor
            .common()
            .tags()
            .iter()
            .filter_map(|tag| tag.tag_id.as_ref().map(|id| (*id, tag)))
            .collect::<HashMap<_, _>>();

        let merged_tags: Vec<Tag> = new_monitor
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

        *new_monitor.common_mut().tags_mut() = merged_tags;
    }

    serde_merge::omerge(current, new).unwrap()
}
