use crate::app_state::AppState;
use crate::entity::{merge_entities, Entity};
use crate::kuma::get_managed_entities;
use crate::name::{EntitySelector, Name};
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
use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct Sync {
    app_state: Arc<AppState>,
    auth_token: Option<String>,
    sources: Vec<Box<dyn Source>>,
    client: Option<Arc<Client>>,
}

impl Sync {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let state = Arc::new(AppState::new(config)?);
        Ok(Self {
            app_state: state.clone(),
            auth_token: state.config.kuma.auth_token.clone(),
            sources: crate::sources::get_sources(state),
            client: None,
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

    async fn delete_entity(&self, kuma: &Client, name: &str, entity: &Entity) -> Result<()> {
        if let Some(selector) = Self::create_entity_selector(name.to_owned(), entity)? {
            self.delete_entity_by_id(kuma, selector).await?;
        }

        Ok(())
    }

    async fn delete_entity_by_id(&self, kuma: &Client, entity: EntitySelector) -> Result<()> {
        info!("Deleting {}: {}", entity.type_name(), entity.name());
        match &entity {
            EntitySelector::Monitor(_, id) => kuma.delete_monitor(*id).await?,
            EntitySelector::Notification(_, id) => kuma.delete_notification(*id).await?,
            EntitySelector::DockerHost(_, id) => kuma.delete_docker_host(*id).await?,
            EntitySelector::Tag(_, id) => kuma.delete_tag(*id).await?,
            EntitySelector::StatusPage(_, id) => kuma.delete_status_page(id).await?,
        }

        self.app_state.db.remove_id(entity.into())?;

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
                    "Recreating entity because type changed: {} ({:?} -> {:?})",
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

    fn create_entity_selector(name: String, entity: &Entity) -> Result<Option<EntitySelector>> {
        Ok(match entity {
            Entity::Monitor(monitor) => monitor
                .common()
                .id()
                .map(|id| EntitySelector::Monitor(name, id)),
            Entity::DockerHost(docker_host) => docker_host
                .id
                .map(|id| EntitySelector::DockerHost(name, id)),
            Entity::Notification(notification) => notification
                .id
                .map(|id| EntitySelector::Notification(name, id)),
            Entity::Tag(tag) => tag.tag_id.map(|id| EntitySelector::Tag(name, id)),
            Entity::StatusPage(status_page) => status_page
                .slug
                .as_ref()
                .map(|id| EntitySelector::StatusPage(name, id.clone())),
        })
    }

    async fn get_connection(&mut self) -> Result<Arc<Client>> {
        if self.client.is_none() {
            let kuma_config = kuma_client::Config {
                auth_token: self.auth_token.clone(),
                ..self.app_state.config.kuma.clone()
            };
            let kuma = Client::connect(kuma_config).await?;
            self.client = Some(Arc::new(kuma));
        }

        Ok(self.client.as_ref().unwrap().clone())
    }

    async fn do_sync(&mut self) -> Result<()> {
        let kuma = self.get_connection().await?;

        crate::migrations::migrate(&self.app_state, &kuma).await?;

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
            let _ = self
                .create_entity(&kuma, id, entity)
                .await
                .log_warn(std::module_path!(), |e| {
                    format!("Failed to create '{}': {}", id, e)
                });
        }

        for (id, current, new) in to_update {
            let _ = self
                .update_entity(&kuma, id, current, new)
                .await
                .log_warn(std::module_path!(), |e| {
                    format!("Failed to update '{}': {}", id, e)
                });
        }

        if self.app_state.config.on_delete == DeleteBehavior::Delete {
            let delete_at = chrono::Utc::now()
                + chrono::Duration::seconds(self.app_state.config.delete_grace_period as i64);

            for selector in to_delete.iter().flat_map(|(name, entity)| {
                Self::create_entity_selector((*name).to_owned(), entity)
                    .ok()
                    .flatten()
            }) {
                self.app_state
                    .db
                    .request_to_delete(selector.clone(), delete_at)
                    .log_warn(std::module_path!(), |e| {
                        format!(
                            "Failed to enqueue deletion of {} {}: {}",
                            selector.type_name(),
                            selector.name(),
                            e
                        )
                    })?;
            }
        }

        for entity in self.app_state.db.get_entities_to_delete()? {
            let name = entity.name().to_owned();

            // Entity reappeared, do not delete
            if !to_delete.iter().any(|(name, _)| name == name) {
                continue;
            }

            let _ = self
                .delete_entity_by_id(&kuma, entity)
                .await
                .log_warn(std::module_path!(), |e| {
                    format!("Failed to delete '{}': {}", name, e)
                });
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
