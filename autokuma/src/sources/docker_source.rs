use crate::{
    app_state::AppState,
    config,
    entity::{get_entities_from_labels, Entity},
    error::Result,
    kuma::get_kuma_labels,
    sources::source::Source,
};
use async_trait::async_trait;
use bollard::{
    container::ListContainersOptions,
    models::SystemInfo,
    service::{ContainerSummary, ListServicesOptions, Service},
    Docker,
};
use itertools::Itertools;
use kuma_client::util::ResultLogger;
use std::{collections::HashMap, env};

async fn get_kuma_containers(state: &AppState, docker: &Docker) -> Result<Vec<ContainerSummary>> {
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

async fn get_kuma_services(state: &AppState, docker: &Docker) -> Result<Vec<Service>> {
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

fn get_entities_from_containers(
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

fn get_entities_from_services(
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

pub struct DockerSource {}

#[async_trait]
impl Source for DockerSource {
    async fn get_entities(&mut self, state: &AppState) -> Result<Vec<(String, Entity)>> {
        if !state.config.docker.enabled {
            return Ok(vec![]);
        }

        let docker_hosts = state
            .config
            .docker
            .hosts
            .clone()
            .map(|f| f.into_iter().map(Some).collect::<Vec<_>>())
            .unwrap_or_else(|| {
                vec![state
                    .config
                    .docker
                    .socket_path
                    .as_ref()
                    .and_then(|path| Some(format!("unix://{}", path)))]
            });

        let mut entities = vec![];

        for docker_host in docker_hosts {
            if let Some(docker_host) = &docker_host {
                env::set_var("DOCKER_HOST", docker_host);
            }

            let docker = Docker::connect_with_defaults().log_warn(std::module_path!(), |_| {
                format!(
                    "Using DOCKER_HOST={}",
                    env::var("DOCKER_HOST").unwrap_or_else(|_| "None".to_owned())
                )
            })?;

            let system_info: bollard::secret::SystemInfo = docker.info().await.unwrap_or_default();

            if state.config.docker.source == config::DockerSource::Containers
                || state.config.docker.source == config::DockerSource::Both
            {
                let containers = get_kuma_containers(&state, &docker).await?;
                entities.extend(get_entities_from_containers(
                    &state,
                    &system_info,
                    &containers,
                )?);
            }

            if state.config.docker.source == config::DockerSource::Services
                || state.config.docker.source == config::DockerSource::Both
            {
                let services = get_kuma_services(&state, &docker).await?;
                entities.extend(get_entities_from_services(&state, &system_info, &services)?);
            }
        }

        Ok(entities)
    }
}
