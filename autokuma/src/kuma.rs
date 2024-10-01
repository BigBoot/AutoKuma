use crate::{app_state::AppState, entity::Entity, error::Result, util::fill_templates};
use kuma_client::{
    docker_host::DockerHost, monitor::Monitor, notification::Notification, tag::TagDefinition,
    Client,
};
use std::collections::HashMap;

pub fn get_kuma_labels(
    state: &AppState,
    labels: Option<&HashMap<String, String>>,
    template_values: &tera::Context,
) -> Result<Vec<(String, String)>> {
    labels.as_ref().map_or_else(
        || Ok(vec![]),
        |labels| {
            labels
                .iter()
                .filter(|(key, _)| {
                    key.starts_with(&format!("{}.", state.config.docker.label_prefix))
                })
                .map(|(key, value)| {
                    fill_templates(
                        key.trim_start_matches(&format!("{}.", state.config.docker.label_prefix)),
                        &template_values,
                    )
                    .map(|key| (key, value.to_owned()))
                })
                .collect::<Result<Vec<_>>>()
        },
    )
}

async fn get_managed_docker_hosts(
    state: &AppState,
    kuma: &Client,
) -> Result<HashMap<String, DockerHost>> {
    let map = state
        .db
        .get_docker_hosts()?
        .into_iter()
        .map(|(key, value)| (value, key))
        .collect::<HashMap<_, _>>();

    Ok(kuma
        .get_docker_hosts()
        .await?
        .into_iter()
        .filter_map(|docker_host| {
            map.get(&docker_host.id.unwrap_or(-1))
                .map(|id| (id.to_owned(), docker_host))
        })
        .collect::<HashMap<_, _>>())
}

async fn get_managed_notification_providers(
    state: &AppState,
    kuma: &Client,
) -> Result<HashMap<String, Notification>> {
    let map = state
        .db
        .get_notifications()?
        .into_iter()
        .map(|(key, value)| (value, key))
        .collect::<HashMap<_, _>>();

    Ok(kuma
        .get_notifications()
        .await?
        .into_iter()
        .filter_map(|docker_host| {
            map.get(&docker_host.id.unwrap_or(-1))
                .map(|id| (id.to_owned(), docker_host))
        })
        .collect::<HashMap<_, _>>())
}

async fn get_managed_tags(
    state: &AppState,
    kuma: &Client,
) -> Result<HashMap<String, TagDefinition>> {
    let map = state
        .db
        .get_tags()?
        .into_iter()
        .map(|(key, value)| (value, key))
        .collect::<HashMap<_, _>>();

    Ok(kuma
        .get_tags()
        .await?
        .into_iter()
        .filter_map(|tag| {
            map.get(&tag.tag_id.unwrap_or(-1))
                .map(|id| (id.to_owned(), tag))
        })
        .collect::<HashMap<_, _>>())
}

async fn get_managed_monitors(state: &AppState, kuma: &Client) -> Result<HashMap<String, Monitor>> {
    let map = state
        .db
        .get_monitors()?
        .into_iter()
        .map(|(key, value)| (value, key))
        .collect::<HashMap<_, _>>();

    Ok(kuma
        .get_monitors()
        .await?
        .into_iter()
        .filter_map(|(_, monitor)| {
            map.get(&monitor.common().id().unwrap_or(-1))
                .map(|id| (id.to_owned(), monitor))
        })
        .collect::<HashMap<_, _>>())
}

pub async fn get_managed_entities(
    state: &AppState,
    kuma: &Client,
) -> Result<HashMap<String, Entity>> {
    Ok(get_managed_monitors(&state, &kuma)
        .await?
        .into_iter()
        .map(|(id, monitor)| (id, Entity::Monitor(monitor)))
        .chain(
            get_managed_docker_hosts(&state, &kuma)
                .await?
                .into_iter()
                .map(|(id, host)| (id, Entity::DockerHost(host))),
        )
        .chain(
            get_managed_notification_providers(&state, &kuma)
                .await?
                .into_iter()
                .map(|(id, notification)| (id, Entity::Notification(notification))),
        )
        .chain(
            get_managed_tags(&state, &kuma)
                .await?
                .into_iter()
                .map(|(id, tag)| (id, Entity::Tag(tag))),
        )
        .collect::<HashMap<_, _>>())
}
