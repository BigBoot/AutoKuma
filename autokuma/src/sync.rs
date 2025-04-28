use crate::app_state::AppState;
use crate::entity::{merge_entities, Entity};
use crate::kuma::get_managed_entities;
use crate::name::Name;
use crate::{
    config::{Config, DeleteBehavior},
    error::{KumaError, Result},
    sources::source::Source,
};
use futures_util::FutureExt;
use itertools::Itertools;
use kuma_client::{util::ResultLogger, Client};
use log::{debug, error, info, trace, warn};
use std::collections::HashSet;
use std::{collections::HashMap, env, sync::Arc, time::Duration};

pub struct Sync {
    app_state: Arc<AppState>,
    auth_token: Option<String>,
    sources: Vec<Box<dyn Source>>,
}

impl Sync {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let state = Arc::new(AppState::new(config)?);
        Ok(Self {
            app_state: state.clone(),
            auth_token: state.config.kuma.auth_token.clone(),
            sources: crate::sources::get_sources(state),
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
            Entity::StatusPage(status_page) => {
                let db_id = kuma
                    .add_status_page(status_page)
                    .await?
                    .slug
                    .ok_or_else(|| {
                        KumaError::CommunicationError(
                            "Did not receive an id from Uptime Kuma".to_owned(),
                        )
                    })?;

                self.app_state
                    .db
                    .store_id(Name::StatusPage(id.clone()), db_id)?;
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
            Entity::StatusPage(status_page) => {
                if let Some(slug) = &status_page.slug {
                    kuma.delete_status_page(slug).await?;
                    self.app_state.db.remove_id(Name::StatusPage(id.clone()))?;
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

    async fn do_sync(&mut self) -> Result<()> {
        let kuma_config = kuma_client::Config {
            auth_token: self.auth_token.clone(),
            ..self.app_state.config.kuma.clone()
        };
        let kuma = Client::connect(kuma_config).await?;

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
                        "Migration required, but AUTOKUMA__MIGRATE is not set to 'true', refusing to continue to avoid data loss. Please read the CHANGELOG and then set AUTOKUMA__MIGRATE=true to continue."
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
            &kuma
                .get_status_pages()
                .await?
                .into_iter()
                .filter_map(|(_, status_page)| status_page.slug)
                .collect::<HashSet<_>>(),
        )?;

        if let Some(auth_token) = kuma.get_auth_token().await {
            self.auth_token = Some(auth_token);
        }

        let current_entities = get_managed_entities(&self.app_state, &kuma).await?;

        let mut new_entities: HashMap<String, Entity> = HashMap::new();

        for source in &mut self.sources {
            trace!("Querying source: {}", source.name());
            let entities = source.get_entities().await?;
            trace!("Got {} entities from source", entities.len());
            new_entities.extend(entities);
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

    async fn init(&mut self) -> Result<()> {
        for source in &mut self.sources {
            source.init().await?;
        }

        Ok(())
    }

    pub async fn run(&mut self) {
        if let Err(err) = self.init().await {
            error!("Encountered error during init: {}", err);
            return;
        }

        async fn shutdown_signal() {
            futures_util::future::select(
                tokio::signal::ctrl_c().map(|_| ()).boxed(),
                #[cfg(unix)]
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .unwrap()
                    .recv()
                    .map(|_| ())
                    .boxed(),
                #[cfg(not(unix))]
                futures_util::future::pending::<()>(),
            )
            .await;
        }

        loop {
            if let Err(err) = self.do_sync().await {
                warn!("Encountered error during sync: {}", err);
            }

            match futures_util::future::select(
                tokio::time::sleep(Duration::from_secs_f64(self.app_state.config.sync_interval))
                    .boxed(),
                shutdown_signal().boxed(),
            )
            .await
            {
                futures_util::future::Either::Left(_) => {}
                futures_util::future::Either::Right(_) => break,
            }
        }

        log::info!("Shutting down...");
        for source in &mut self.sources {
            _ = source.shutdown().await.log_error(std::module_path!(), |e| {
                format!("Failed to gracefully shutdown source: {}", e)
            });
        }
    }
}
