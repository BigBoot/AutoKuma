use crate::{
    app_state::AppState,
    config::{self, DockerHostConfig},
    entity::{Entity, get_entities_from_labels},
    error::Result,
    kuma::get_kuma_labels,
    sources::source::Source,
};
use async_trait::async_trait;
use bollard::{
    models::SystemInfo,
    query_parameters::{ListContainersOptionsBuilder, ListServicesOptionsBuilder},
    service::{ContainerSummary, Service},
    Docker,
};
use itertools::Itertools;
use kuma_client::util::ResultLogger;
use regex::Regex;
use std::{collections::HashMap, env, sync::Arc};

// Helper function to extract container name, flattening the Option chain
fn get_container_name(container: &ContainerSummary) -> Option<&str> {
    container
        .names
        .as_ref()?
        .first()
        .map(|name| name.trim_start_matches("/"))
}

// Helper function to check if container should be excluded using pre-compiled regexes
fn is_excluded_by_patterns(container_name: &str, compiled_patterns: &[Regex]) -> bool {
    for regex in compiled_patterns {
        if regex.is_match(container_name) {
            log::debug!(
                "Excluding container '{}' due to exclusion pattern",
                container_name
            );
            return true;
        }
    }
    false
}

async fn get_kuma_containers(
    state: Arc<AppState>,
    docker: &Docker,
) -> Result<Vec<ContainerSummary>> {
    Ok(docker
        .list_containers(Some(ListContainersOptionsBuilder::new().all(true).build()))
        .await
        .log_warn(std::module_path!(), |_| {
            format!(
                "Using DOCKER_HOST={}",
                env::var("DOCKER_HOST").unwrap_or_else(|_| "None".to_owned())
            )
        })?
        .into_iter()
        .filter(|container| {
            // Check if container has required labels
            let has_kuma_labels = container.labels.as_ref().map_or_else(
                || false,
                |labels| {
                    labels.keys().any(|key| {
                        key.starts_with(&format!("{}.", state.config.docker.label_prefix))
                            || state.config.snippets.contains_key(&format!("!{}", key))
                    })
                },
            );

            if !has_kuma_labels {
                return false;
            }

            // Check if container should be excluded using pre-compiled regexes
            if !state.compiled_exclusion_patterns.is_empty() {
                if let Some(container_name) = get_container_name(container) {
                    if is_excluded_by_patterns(container_name, &state.compiled_exclusion_patterns) {
                        return false;
                    }
                }
            }

            true
        })
        .collect::<Vec<_>>())
}

async fn get_kuma_services(state: Arc<AppState>, docker: &Docker) -> Result<Vec<Service>> {
    Ok(docker
        .list_services(Some(ListServicesOptionsBuilder::new().build()))
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
                                    || state.config.snippets.contains_key(&format!("!{}", key))
                            })
                        },
                    )
                },
            )
        })
        .collect::<Vec<_>>())
}

fn get_entities_from_containers(
    state: Arc<AppState>,
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

            let kuma_labels = get_kuma_labels(&state, container.labels.as_ref(), &template_values)?;

            get_entities_from_labels(state.clone(), kuma_labels, &template_values)
        })
        .flatten_ok()
        .try_collect()
}

fn get_entities_from_services(
    state: Arc<AppState>,
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

            let kuma_labels = get_kuma_labels(&state, labels, &template_values)?;

            get_entities_from_labels(state.clone(), kuma_labels, &template_values)
        })
        .flatten_ok()
        .try_collect()
}

pub struct DockerSource {
    state: Arc<AppState>,
}

#[async_trait]
impl Source for DockerSource {
    fn name(&self) -> &'static str {
        "Docker"
    }

    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    async fn get_entities(&mut self) -> Result<Vec<(String, Entity)>> {
        if !self.state.config.docker.enabled {
            return Ok(vec![]);
        }

        let docker_hosts = self
            .state
            .config
            .docker
            .hosts
            .clone()
            .map(|hosts| {
                hosts
                    .into_iter()
                    .map(|host| Some(host))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                vec![self
                    .state
                    .config
                    .docker
                    .socket_path
                    .as_ref()
                    .and_then(|path| Some(DockerHostConfig{url: format!("unix://{}", path), tls_verify: None, tls_cert_path: None}))]
            });

        let mut entities = vec![];

        for docker_host in docker_hosts {
            if let Some(docker_host) = &docker_host {
                env::set_var("DOCKER_HOST", docker_host.url.clone());
                if let Some(tls_verify) = docker_host.tls_verify {
                    env::set_var("DOCKER_TLS_VERIFY", tls_verify.to_string());
                }
                if let Some(tls_cert_path) = &docker_host.tls_cert_path {
                    env::set_var("DOCKER_CERT_PATH", tls_cert_path);
                }
            }

            let docker = Docker::connect_with_defaults().log_warn(std::module_path!(), |_| {
                format!(
                    "Using DOCKER_HOST={}, DOCKER_TLS_VERIFY={}, DOCKER_CERT_PATH={}",
                    env::var("DOCKER_HOST").unwrap_or_else(|_| "None".to_owned()),
                    env::var("DOCKER_TLS_VERIFY").unwrap_or_else(|_| "None".to_owned()),
                    env::var("DOCKER_CERT_PATH").unwrap_or_else(|_| "None".to_owned())
                )
            })?;

            let system_info: bollard::secret::SystemInfo = docker.info().await.unwrap_or_default();

            if self.state.config.docker.source == config::DockerSource::Containers
                || self.state.config.docker.source == config::DockerSource::Both
            {
                let containers = get_kuma_containers(self.state.clone(), &docker).await?;
                entities.extend(get_entities_from_containers(
                    self.state.clone(),
                    &system_info,
                    &containers,
                )?);
            }

            if self.state.config.docker.source == config::DockerSource::Services
                || self.state.config.docker.source == config::DockerSource::Both
            {
                let services = get_kuma_services(self.state.clone(), &docker).await?;
                entities.extend(get_entities_from_services(
                    self.state.clone(),
                    &system_info,
                    &services,
                )?);
            }
        }

        Ok(entities)
    }
}

impl DockerSource {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}
