use crate::{
    app_state::AppState,
    entity::{get_entities_from_labels, Entity},
    error::Result,
    kuma::get_kuma_labels,
};
use bollard::{
    container::ListContainersOptions,
    models::SystemInfo,
    service::{ContainerSummary, ListServicesOptions, Service},
    Docker,
};
use itertools::Itertools;
use kuma_client::util::ResultLogger;
use std::{collections::HashMap, env};

pub async fn get_kuma_containers(
    state: &AppState,
    docker: &Docker,
) -> Result<Vec<ContainerSummary>> {
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
                        key.starts_with(&format!("{}.", state.config.docker.label_prefix))
                    })
                },
            )
        })
        .collect::<Vec<_>>())
}

pub async fn get_kuma_services(state: &AppState, docker: &Docker) -> Result<Vec<Service>> {
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
                                key.starts_with(&format!("{}.", state.config.docker.label_prefix))
                            })
                        },
                    )
                },
            )
        })
        .collect::<Vec<_>>())
}

pub fn get_entities_from_containers(
    state: &AppState,
    system_info: &SystemInfo,
    containers: &Vec<ContainerSummary>,
) -> Result<HashMap<String, Entity>> {
    containers
        .into_iter()
        .map(|container| {
            let mut template_values = tera::Context::new();
            template_values.insert("container_id", &container.id);
            template_values.insert("image_id", &container.image_id);
            template_values.insert("image", &container.image);
            template_values.insert(
                "container_name",
                &container
                    .names
                    .as_ref()
                    .and_then(|names| names.first().map(|s| s.trim_start_matches("/").to_owned())),
            );

            template_values.insert("container", &container);
            template_values.insert("system_info", system_info);

            let kuma_labels = get_kuma_labels(state, container.labels.as_ref(), &template_values)?;

            get_entities_from_labels(state, kuma_labels, &template_values)
        })
        .flatten_ok()
        .try_collect()
}

pub fn get_entities_from_services(
    state: &AppState,
    system_info: &SystemInfo,
    services: &Vec<Service>,
) -> Result<HashMap<String, Entity>> {
    services
        .into_iter()
        .map(|service| {
            let mut template_values = tera::Context::new();

            template_values.insert("service", &service);
            template_values.insert("system_info", system_info);

            let spec = service.spec.as_ref();
            let labels = spec.and_then(|spec| spec.labels.as_ref());

            let kuma_labels = get_kuma_labels(state, labels, &template_values)?;

            get_entities_from_labels(state, kuma_labels, &template_values)
        })
        .flatten_ok()
        .try_collect()
}
