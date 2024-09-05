use crate::{app_state::AppState, entity::Entity, error::Result, util::fill_templates};
use kuma_client::{docker_host::DockerHost, monitor::Monitor, notification::Notification, Client};
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

async fn get_managed_docker_hosts(kuma: &Client) -> Result<HashMap<String, DockerHost>> {
    Ok(kuma
        .get_docker_hosts()
        .await?
        .into_iter()
        .filter_map(|docker_host| docker_host.name.clone().map(|name| (name, docker_host)))
        .filter(|(name, _)| name.starts_with("autokuma__"))
        .map(|(name, docker_host)| {
            (
                name.trim_start_matches("autokuma__").to_owned(),
                docker_host,
            )
        })
        .collect::<HashMap<_, _>>())
}

async fn get_managed_notification_providers(
    kuma: &Client,
) -> Result<HashMap<String, Notification>> {
    Ok(kuma
        .get_notifications()
        .await?
        .into_iter()
        .filter_map(|notification| notification.name.clone().map(|name| (name, notification)))
        .filter(|(name, _)| name.starts_with("autokuma__"))
        .map(|(name, docker_host)| {
            (
                name.trim_start_matches("autokuma__").to_owned(),
                docker_host,
            )
        })
        .collect::<HashMap<_, _>>())
}

async fn get_managed_monitors(state: &AppState, kuma: &Client) -> Result<HashMap<String, Monitor>> {
    Ok(kuma
        .get_monitors()
        .await?
        .into_iter()
        .filter_map(|(_, monitor)| {
            match monitor
                .common()
                .tags()
                .iter()
                .filter(|tag| tag.name.as_deref() == Some(&state.config.tag_name))
                .find_map(|tag| tag.value.as_ref())
            {
                Some(id) => Some((id.to_owned(), monitor)),
                None => None,
            }
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
            get_managed_docker_hosts(&kuma)
                .await?
                .into_iter()
                .map(|(id, host)| (id, Entity::DockerHost(host))),
        )
        .chain(
            get_managed_notification_providers(&kuma)
                .await?
                .into_iter()
                .map(|(id, notification)| (id, Entity::Notification(notification))),
        )
        .collect::<HashMap<_, _>>())
}
