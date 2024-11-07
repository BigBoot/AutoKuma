use crate::{
    app_state::AppState,
    entity::{get_entity_from_value, Entity},
    error::{Error, K8SError, Result},
    sources::source::Source,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use kube::{
    api::ListParams,
    runtime::{
        controller::Action,
        finalizer::{finalizer, Event as Finalizer},
        watcher::Config as WatcherConfig,
        Controller,
    },
    Api, Client, CustomResource, ResourceExt,
};
use kuma_client::util::ResultLogger;
use log::{error, info, trace, warn};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub static ENTITY_FINALIZER: &str = "entity.autokuma.bigboot.dev";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[cfg_attr(test, derive(Default))]
#[kube(
    kind = "KumaEntity",
    group = "autokuma.bigboot.dev",
    version = "v1",
    namespaced
)]
pub struct KumaEntitySpec {
    pub config: serde_json::Map<String, serde_json::Value>,
}

pub struct Context {
    pub client: Client,
    pub entities: Arc<Mutex<BTreeMap<String, Entity>>>,
    pub state: Arc<AppState>,
}

async fn reconcile(entity: Arc<KumaEntity>, ctx: Arc<Context>) -> Result<Action> {
    let ns = entity.namespace().unwrap();
    let api: Api<KumaEntity> = Api::namespaced(ctx.client.clone(), &ns);

    trace!("Reconciling Entity \"{}\" in {}", entity.name_any(), ns);
    finalizer(&api, ENTITY_FINALIZER, entity, |event| async {
        match event {
            Finalizer::Apply(doc) => doc.reconcile(ctx.clone()).await,
            Finalizer::Cleanup(doc) => doc.cleanup(ctx.clone()).await,
        }
    })
    .await
    .map_err(|e| Error::K8S(K8SError::FinalizerError(Box::new(e))))
}

fn error_policy(_entity: Arc<KumaEntity>, error: &Error, _ctx: Arc<Context>) -> Action {
    warn!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5 * 60))
}

impl KumaEntity {
    async fn reconcile(&self, ctx: Arc<Context>) -> Result<Action> {
        let id = self.name_any();
        let entity = get_entity_from_value(
            ctx.state.clone(),
            id.clone(),
            self.spec.config.clone().into(),
            tera::Context::new(),
        )?;

        let mut entities = ctx.entities.lock().await;
        entities.insert(id, entity);

        Ok(Action::requeue(Duration::from_secs(5 * 60)))
    }

    async fn cleanup(&self, ctx: Arc<Context>) -> Result<Action> {
        let name = self.name_any();
        let mut entities = ctx.entities.lock().await;
        entities.remove(&name);

        Ok(Action::await_change())
    }
}

pub struct KubernetesSource {
    state: Arc<AppState>,
    shutdown: Option<tokio::sync::mpsc::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
    entities: Arc<Mutex<BTreeMap<String, Entity>>>,
}

#[async_trait]
impl Source for KubernetesSource {
    fn name(&self) -> &'static str {
        "Kubernetes"
    }

    async fn init(&mut self) -> Result<()> {
        let client = Client::try_default()
            .await
            .expect("failed to create kube Client");

        let docs = Api::<KumaEntity>::all(client.clone());
        if let Err(e) = docs.list(&ListParams::default().limit(1)).await {
            error!("CRD is not queryable; {e:?}. Is the CRD installed?");
            info!("Installation: cargo run --bin crdgen | kubectl apply -f -");
            std::process::exit(1);
        }

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);
        let state = self.state.clone();
        let entities = self.entities.clone();
        self.task = Some(tokio::spawn(async move {
            Controller::new(docs, WatcherConfig::default().any_semantic())
                .graceful_shutdown_on(async move { shutdown_rx.recv().await.unwrap_or(()) })
                .run(
                    reconcile,
                    error_policy,
                    Arc::new(Context {
                        client,
                        entities,
                        state,
                    }),
                )
                .filter_map(|x| async move { std::result::Result::ok(x) })
                .for_each(|_| futures_util::future::ready(()))
                .await;
        }));

        self.shutdown = Some(shutdown_tx);
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown) = &self.shutdown {
            _ = shutdown.send(()).await.log_error(std::module_path!(), |e| {
                format!("Failed to shutdown kube client: {}", e)
            });
        }

        if let Some(task) = self.task.take() {
            _ = task.await.log_error(std::module_path!(), |e| {
                format!("Failed to await kube client shutdown: {}", e)
            });
        }

        Ok(())
    }

    async fn get_entities(&mut self) -> Result<Vec<(String, Entity)>> {
        let entities = self.entities.lock().await;

        Ok(entities
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
}

impl KubernetesSource {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            shutdown: None,
            task: None,
            entities: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}
