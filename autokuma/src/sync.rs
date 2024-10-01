use crate::app_state::AppState;
use crate::docker_source::{
    get_entities_from_containers, get_entities_from_services, get_kuma_containers,
    get_kuma_services,
};
use crate::entity::{merge_entities, Entity};
use crate::file_source::get_entity_from_file;
use crate::kuma::get_managed_entities;
use crate::name::Name;
use crate::{
    config::{Config, DeleteBehavior, DockerSource},
    error::{Error, KumaError, Result},
    util::ResultLogger,
};
use bollard::Docker;
use itertools::Itertools;
use kuma_client::Client;
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::{collections::HashMap, env, sync::Arc, time::Duration};

pub struct Sync {
    app_state: AppState,
}

impl Sync {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self {
            app_state: AppState::new(config)?,
        })
    }

    async fn create_entity(&self, kuma: &Client, id: &String, entity: &Entity) -> Result<()> {
        info!("Creating new {}: {}", entity.entity_type(), id);
        match entity.clone() {
            Entity::Monitor(monitor) => {
                match kuma.add_monitor(monitor).await {
                    Ok(monitor) => {
                        let db_id = monitor.common().id().ok_or_else(|| {
                            KumaError::CommunicationError(
                                "Did not receive an id from Uptime Kuma".to_owned(),
                            )
                        })?;

                        self.app_state
                            .db
                            .store_id(Name::Monitor(id.clone()), db_id)?;

                        Ok(())
                    }
                    Err(err) => Err(err),
                }?;
            }
            Entity::DockerHost(docker_host) => {
                let db_id = kuma.add_docker_host(docker_host).await?.id.ok_or_else(|| {
                    KumaError::CommunicationError(
                        "Did not receive an id from Uptime Kuma".to_owned(),
                    )
                })?;

                self.app_state
                    .db
                    .store_id(Name::DockerHost(id.clone()), db_id)?;
            }
            Entity::Notification(notification) => {
                let db_id = kuma
                    .add_notification(notification)
                    .await?
                    .id
                    .ok_or_else(|| {
                        KumaError::CommunicationError(
                            "Did not receive an id from Uptime Kuma".to_owned(),
                        )
                    })?;

                self.app_state
                    .db
                    .store_id(Name::Notification(id.clone()), db_id)?;
            }
            Entity::Tag(tag) => {
                let db_id = kuma.add_tag(tag).await?.tag_id.ok_or_else(|| {
                    KumaError::CommunicationError(
                        "Did not receive an id from Uptime Kuma".to_owned(),
                    )
                })?;

                self.app_state.db.store_id(Name::Tag(id.clone()), db_id)?;
            }
        }

        Ok(())
    }

    async fn delete_entity(&self, kuma: &Client, id: &String, entity: &Entity) -> Result<()> {
        info!("Deleting {}: {}", entity.entity_type(), id);
        match entity {
            Entity::Monitor(monitor) => {
                if let Some(db_id) = monitor.common().id() {
                    kuma.delete_monitor(*db_id).await?;
                    self.app_state.db.remove_id(Name::Monitor(id.clone()))?;
                }
            }
            Entity::DockerHost(docker_host) => {
                if let Some(db_id) = docker_host.id {
                    kuma.delete_docker_host(db_id).await?;
                    self.app_state.db.remove_id(Name::DockerHost(id.clone()))?;
                }
            }
            Entity::Notification(notification) => {
                if let Some(db_id) = notification.id {
                    kuma.delete_notification(db_id).await?;
                    self.app_state
                        .db
                        .remove_id(Name::Notification(id.clone()))?;
                }
            }
            Entity::Tag(tag) => {
                if let Some(db_id) = tag.tag_id {
                    kuma.delete_tag(db_id).await?;
                    self.app_state.db.remove_id(Name::Tag(id.clone()))?;
                }
            }
        }

        Ok(())
    }

    async fn update_entity(
        &self,
        kuma: &Client,
        id: &String,
        current: &Entity,
        new: &Entity,
    ) -> Result<()> {
        let merge = merge_entities(&current, &new, None);

        if current != &merge {
            debug!(
                "\n======= OLD =======\n{}\n===================\n\n======= NEW =======\n{}\n===================", 
                serde_json::to_string_pretty(&current).unwrap(),
                serde_json::to_string_pretty(&merge).unwrap()
            );

            if current.entity_type() != new.entity_type() {
                info!(
                    "Recreating entity because type changed: {} ({} -> {})",
                    id,
                    current.entity_type(),
                    new.entity_type()
                );
                self.delete_entity(kuma, id, &current).await?;
                self.create_entity(kuma, id, &new).await?;
                return Ok(());
            }

            info!("Updating {}: {}", new.entity_type(), id);

            match (merge, current) {
                (Entity::Monitor(merge), Entity::Monitor(_)) => {
                    kuma.edit_monitor(merge).await?;
                }
                (Entity::DockerHost(merge), Entity::DockerHost(_)) => {
                    kuma.edit_docker_host(merge).await?;
                }
                (Entity::Notification(merge), Entity::Notification(_)) => {
                    kuma.edit_notification(merge).await?;
                }
                (Entity::Tag(merge), Entity::Tag(_)) => {
                    kuma.edit_tag(merge).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn do_sync(&self) -> Result<()> {
        let kuma = Client::connect(self.app_state.config.kuma.clone()).await?;

        if self.app_state.db.get_version()? == 0 {
            let autokuma_tag = kuma
                .get_tags()
                .await?
                .iter()
                .find(|x| {
                    x.name
                        .as_ref()
                        .is_some_and(|name| name == &self.app_state.config.tag_name)
                })
                .map(|tag| tag.tag_id)
                .flatten();

            if let Some(autokuma_tag) = autokuma_tag {
                if !env::var("AUTOKUMA__MIGRATE").is_ok_and(|x| x == "true") {
                    error!(
                        "Migration required, but AUTOKUMA__MIGRATE is not set to 'true', refusing to continue to avoid data loss. Please read <TODO> and then set AUTOKUMA__MIGRATE=true to continue."
                    );
                    return Ok(());
                }

                let entries = kuma
                    .get_monitors()
                    .await?
                    .iter()
                    .filter_map(|(_, monitor)| {
                        monitor
                            .common()
                            .tags()
                            .iter()
                            .find(|x| x.tag_id == Some(autokuma_tag))
                            .map(|tag| tag.value.clone())
                            .flatten()
                            .map(|name| (name, monitor.common().id().unwrap_or(-1)))
                    })
                    .collect_vec();

                info!("Migrating {} monitors", entries.len());

                for (name, id) in entries {
                    self.app_state.db.store_id(Name::Monitor(name), id)?;
                }

                kuma.delete_tag(autokuma_tag).await?;
            }

            self.app_state.db.set_version(1)?
        }

        self.app_state.db.clean(
            &kuma
                .get_monitors()
                .await?
                .into_iter()
                .filter_map(|(_, monitor)| monitor.common().id().clone())
                .collect::<HashSet<_>>(),
            &kuma
                .get_notifications()
                .await?
                .into_iter()
                .filter_map(|notification| notification.id)
                .collect::<HashSet<_>>(),
            &kuma
                .get_docker_hosts()
                .await?
                .into_iter()
                .filter_map(|docker_host| docker_host.id)
                .collect::<HashSet<_>>(),
            &kuma
                .get_tags()
                .await?
                .into_iter()
                .filter_map(|tag| tag.tag_id)
                .collect::<HashSet<_>>(),
        )?;

        let current_entities = get_managed_entities(&self.app_state, &kuma).await?;

        let mut new_entities: HashMap<String, Entity> = HashMap::new();

        if self.app_state.config.docker.enabled {
            let docker_hosts = self
                .app_state
                .config
                .docker
                .hosts
                .clone()
                .map(|f| f.into_iter().map(Some).collect::<Vec<_>>())
                .unwrap_or_else(|| {
                    vec![self
                        .app_state
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

                let system_info: bollard::secret::SystemInfo =
                    docker.info().await.unwrap_or_default();

                if self.app_state.config.docker.source == DockerSource::Containers
                    || self.app_state.config.docker.source == DockerSource::Both
                {
                    let containers = get_kuma_containers(&self.app_state, &docker).await?;
                    new_entities.extend(get_entities_from_containers(
                        &self.app_state,
                        &system_info,
                        &containers,
                    )?);
                }

                if self.app_state.config.docker.source == DockerSource::Services
                    || self.app_state.config.docker.source == DockerSource::Both
                {
                    let services = get_kuma_services(&self.app_state, &docker).await?;
                    new_entities.extend(get_entities_from_services(
                        &self.app_state,
                        &system_info,
                        &services,
                    )?);
                }
            }
        }

        let static_monitor_path = self
            .app_state
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
                        let (id, monitor) =
                            get_entity_from_file(&self.app_state, f.path().to_string_lossy())
                                .await?;
                        new_entities.insert(id, monitor);
                    } else {
                        break;
                    }
                }
            }
        }

        let to_delete = current_entities
            .iter()
            .filter(|(id, _)| !new_entities.contains_key(*id))
            .collect_vec();

        let to_create = new_entities
            .iter()
            .filter(|(id, _)| !current_entities.contains_key(*id))
            .collect_vec();

        let to_update = current_entities
            .keys()
            .filter_map(
                |id| match (current_entities.get(id), new_entities.get(id)) {
                    (Some(current), Some(new)) => Some((id, current, new)),
                    _ => None,
                },
            )
            .collect_vec();

        for (id, entity) in to_create {
            self.create_entity(&kuma, id, entity).await?;
        }

        for (id, current, new) in to_update {
            self.update_entity(&kuma, id, current, new).await?;
        }

        if self.app_state.config.on_delete == DeleteBehavior::Delete {
            for (id, monitor) in to_delete {
                self.delete_entity(&kuma, id, monitor).await?;
            }
        }

        Ok(())
    }

    pub async fn run(&self) {
        loop {
            if let Err(err) = self.do_sync().await {
                warn!("Encountered error during sync: {}", err);
            }
            tokio::time::sleep(Duration::from_secs_f64(self.app_state.config.sync_interval)).await;
        }
    }
}
