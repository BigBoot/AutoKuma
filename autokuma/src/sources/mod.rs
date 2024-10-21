use std::sync::Arc;

use crate::app_state::AppState;

pub mod docker_source;
pub mod file_source;
pub mod source;

#[cfg(feature = "kubernetes")]
pub mod kubernetes_source;

pub fn get_sources(state: Arc<AppState>) -> Vec<Box<dyn source::Source>> {
    let mut sources: Vec<Box<dyn source::Source>> =
        vec![Box::new(file_source::FileSource::new(state.clone()))];

    if state.config.docker.enabled {
        sources.push(Box::new(docker_source::DockerSource::new(state.clone())));
    }

    if state.config.kubernetes.enabled {
        #[cfg(feature = "kubernetes")]
        sources.push(Box::new(kubernetes_source::KubernetesSource::new(
            state.clone(),
        )));
    }

    sources
}
