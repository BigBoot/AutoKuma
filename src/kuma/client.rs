use super::{Error, Event, Monitor, MonitorList, MonitorType, Result, Tag, TagDefinition};
use crate::config::Config;
use crate::util::ResultLogger;
use futures_util::FutureExt;
use itertools::Itertools;
use log::{debug, warn};
use rust_socketio::Payload;
use rust_socketio::{
    asynchronous::{Client as SocketIO, ClientBuilder},
    Event as SocketIOEvent,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::mem;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

type EventArgs = (Event, Value);
type Sender = mpsc::Sender<EventArgs>;

struct Worker {
    config: Arc<Config>,
    socket_io: Arc<Mutex<Option<SocketIO>>>,
    event_sender: Arc<Sender>,
    tags: Arc<Mutex<Vec<TagDefinition>>>,
    monitors: Arc<Mutex<MonitorList>>,
    is_ready: Arc<Mutex<bool>>,
}

impl Worker {
    fn new(config: Arc<Config>, event_sender: Sender) -> Self {
        Worker {
            config: config,
            socket_io: Arc::new(Mutex::new(None)),
            event_sender: (Arc::new(event_sender)),
            tags: Default::default(),
            monitors: Default::default(),
            is_ready: Arc::new(Mutex::new(false)),
        }
    }

    async fn on_monitor_list(&self, monitor_list: MonitorList) -> Result<()> {
        *self.monitors.lock().await = monitor_list;

        let tags = self.get_tags().await;
        *self.tags.lock().await = tags?;
        *self.is_ready.lock().await = true;

        Ok(())
    }

    async fn on_connect(&self) -> Result<()> {
        if let (Some(username), Some(password)) =
            (&self.config.kuma.username, &self.config.kuma.password)
        {
            self.login(username, password, self.config.kuma.mfa_token.clone())
                .await?;
        }

        Ok(())
    }

    async fn on_event(&self, event: Event, payload: Value) -> Result<()> {
        match event {
            Event::MonitorList => {
                self.on_monitor_list(serde_json::from_value(payload).unwrap())
                    .await?
            }
            Event::Connect => self.on_connect().await?,
            _ => {}
        }

        Ok(())
    }

    fn verify_response<T: DeserializeOwned>(
        response: Vec<Value>,
        result_ptr: impl AsRef<str>,
    ) -> Result<T> {
        let json = json!(response);

        if !json
            .pointer("/0/0/ok")
            .ok_or_else(|| {
                Error::InvalidResponse(response.clone(), result_ptr.as_ref().to_owned())
            })?
            .as_bool()
            .unwrap_or_default()
        {
            let error_msg = json
                .pointer("/0/0/msg")
                .unwrap_or_else(|| &json!(null))
                .as_str()
                .unwrap_or_else(|| "Unknown error");

            return Err(Error::ServerError(error_msg.to_owned()));
        }

        json.pointer(&format!("/0/0{}", result_ptr.as_ref()))
            .and_then(|value| serde_json::from_value(value.to_owned()).ok())
            .ok_or_else(|| Error::InvalidResponse(response, result_ptr.as_ref().to_owned()))
    }

    async fn get_tags(&self) -> Result<Vec<TagDefinition>> {
        self.call("getTags", vec![], "/tags", Duration::from_secs(2))
            .await
    }

    async fn call<A, T>(
        &self,
        method: impl Into<String>,
        args: A,
        result_ptr: impl Into<String>,
        timeout: Duration,
    ) -> Result<T>
    where
        A: IntoIterator<Item = Value> + Send + Clone,
        T: DeserializeOwned + Send + 'static,
    {
        let method = method.into();
        let result_ptr: String = result_ptr.into();

        let method = method.clone();
        let args = args.clone();
        let result_ptr = result_ptr.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<T>>(1);

        let lock = self.socket_io.lock().await;
        let socket_io = match &*lock {
            Some(socket_io) => socket_io,
            None => Err(Error::Disconnected)?,
        };

        socket_io
            .emit_with_ack(
                method.clone(),
                Payload::Text(args.into_iter().collect_vec()),
                timeout,
                move |message: Payload, _: SocketIO| {
                    let tx = tx.clone();
                    let result_ptr = result_ptr.clone();
                    async move {
                        _ = match message {
                            Payload::Text(response) => {
                                tx.send(Self::verify_response(response, result_ptr)).await
                            }
                            _ => tx.send(Err(Error::UnsupportedResponse)).await,
                        }
                    }
                    .boxed()
                },
            )
            .await
            .map_err(|e| Error::CommunicationError(e.to_string()))?;

        tokio::time::timeout(Duration::from_secs(3), rx.recv())
            .await
            .map_err(|_| Error::CallTimeout(method.clone()))?
            .ok_or_else(|| Error::CallTimeout(method))?
    }

    pub async fn login(
        &self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        token: Option<String>,
    ) -> Result<bool> {
        self.call(
            "login",
            vec![serde_json::to_value(HashMap::from([
                ("username", json!(username.as_ref())),
                ("password", json!(password.as_ref())),
                ("token", json!(token)),
            ]))
            .unwrap()],
            "/ok",
            Duration::from_secs(2),
        )
        .await
    }

    pub async fn add_tag(&self, tag: TagDefinition) -> Result<TagDefinition> {
        self.call(
            "addTag",
            vec![serde_json::to_value(tag.clone()).unwrap()],
            "/tag",
            Duration::from_secs(2),
        )
        .await
    }

    pub async fn add_monitor_tag(
        &self,
        monitor_id: i32,
        tag_id: i32,
        value: Option<String>,
    ) -> Result<()> {
        let _: bool = self
            .call(
                "addMonitorTag",
                vec![
                    json!(tag_id),
                    json!(monitor_id),
                    json!(value.unwrap_or_default()),
                ],
                "/ok",
                Duration::from_secs(2),
            )
            .await?;

        Ok(())
    }

    pub async fn edit_monitor_tag(
        &self,
        monitor_id: i32,
        tag_id: i32,
        value: Option<String>,
    ) -> Result<()> {
        let _: bool = self
            .call(
                "editMonitorTag",
                vec![
                    json!(tag_id),
                    json!(monitor_id),
                    json!(value.unwrap_or_default()),
                ],
                "/ok",
                Duration::from_secs(2),
            )
            .await?;

        Ok(())
    }

    pub async fn delete_monitor_tag(
        &self,
        monitor_id: i32,
        tag_id: i32,
        value: Option<String>,
    ) -> Result<()> {
        let _: bool = self
            .call(
                "deleteMonitorTag",
                vec![
                    json!(tag_id),
                    json!(monitor_id),
                    json!(value.unwrap_or_default()),
                ],
                "/ok",
                Duration::from_secs(2),
            )
            .await?;

        Ok(())
    }

    pub async fn delete_monitor(&self, monitor_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "deleteMonitor",
                vec![json!(monitor_id)],
                "/ok",
                Duration::from_secs(2),
            )
            .await?;

        Ok(())
    }

    async fn resolve_group(&self, monitor: &mut Monitor) -> Result<bool> {
        if let Some(group_name) = monitor.common().parent_name.clone() {
            monitor.common_mut().parent_name = None;

            if let Some(Some(group_id)) = self
                .monitors
                .lock()
                .await
                .iter()
                .find(|x| {
                    x.1.monitor_type() == MonitorType::Group
                        && x.1.common().tags.iter().any(|tag| {
                            tag.name.as_ref().is_some_and(|tag| tag == "AutoKuma")
                                && tag
                                    .value
                                    .as_ref()
                                    .is_some_and(|tag_value| tag_value == &group_name)
                        })
                })
                .map(|x| x.1.common().id)
            {
                monitor.common_mut().parent = Some(group_id);
            } else {
                return Ok(false);
            }
        } else {
            monitor.common_mut().parent = None;
        }
        return Ok(true);
    }

    async fn update_monitor_tags(&self, monitor_id: i32, tags: &Vec<Tag>) -> Result<()> {
        let new_tags = tags
            .iter()
            .filter_map(|tag| tag.tag_id.and_then(|id| Some((id, tag))))
            .collect::<HashMap<_, _>>();

        if let Some(monitor) = self.monitors.lock().await.get(&monitor_id.to_string()) {
            let current_tags = monitor
                .common()
                .tags
                .iter()
                .filter_map(|tag| tag.tag_id.and_then(|id| Some((id, tag))))
                .collect::<HashMap<_, _>>();

            let duplicates = monitor
                .common()
                .tags
                .iter()
                .duplicates_by(|tag| tag.tag_id)
                .filter_map(|tag| tag.tag_id.as_ref().map(|id| (id, tag)))
                .collect::<HashMap<_, _>>();

            let to_delete = current_tags
                .iter()
                .filter(|(id, _)| !new_tags.contains_key(*id) && !duplicates.contains_key(*id))
                .collect_vec();

            let to_create = new_tags
                .iter()
                .filter(|(id, _)| !current_tags.contains_key(*id))
                .collect_vec();

            let to_update = current_tags
                .keys()
                .filter_map(|id| match (current_tags.get(id), new_tags.get(id)) {
                    (Some(current), Some(new)) => Some((id, current, new)),
                    _ => None,
                })
                .collect_vec();

            for (tag_id, tag) in duplicates {
                self.delete_monitor_tag(monitor_id, *tag_id, tag.value.clone())
                    .await?;
            }

            for (tag_id, tag) in to_delete {
                self.delete_monitor_tag(monitor_id, *tag_id, tag.value.clone())
                    .await?;
            }

            for (tag_id, tag) in to_create {
                self.add_monitor_tag(monitor_id, *tag_id, tag.value.clone())
                    .await?
            }

            for (tag_id, current, new) in to_update {
                if current.value != new.value {
                    self.edit_monitor_tag(monitor_id, *tag_id, new.value.clone())
                        .await?;
                }
            }
        } else {
            for tag in tags {
                if let Some(tag_id) = tag.tag_id {
                    self.add_monitor_tag(monitor_id, tag_id, tag.value.clone())
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn add_monitor(&self, mut monitor: Monitor) -> Result<Monitor> {
        let mut tags = vec![];
        mem::swap(&mut tags, &mut monitor.common_mut().tags);

        if !self.resolve_group(&mut monitor).await? {
            return Ok(monitor);
        }

        if let Some(notifications) = &mut monitor.common_mut().notification_id_list {
            notifications.clear();
        }

        let id: i32 = self
            .call(
                "add",
                vec![serde_json::to_value(&monitor).unwrap()],
                "/monitorID",
                Duration::from_secs(2),
            )
            .await?;

        self.update_monitor_tags(id, &tags).await?;

        monitor.common_mut().id = Some(id);
        monitor.common_mut().tags = tags;

        self.monitors
            .lock()
            .await
            .insert(id.to_string(), monitor.clone());

        Ok(monitor)
    }

    pub async fn edit_monitor(&self, mut monitor: Monitor) -> Result<Monitor> {
        if !self.resolve_group(&mut monitor).await? {
            return Ok(monitor);
        }

        let mut tags = vec![];
        mem::swap(&mut tags, &mut monitor.common_mut().tags);

        let id: i32 = self
            .call(
                "editMonitor",
                vec![serde_json::to_value(&monitor).unwrap()],
                "/monitorID",
                Duration::from_secs(2),
            )
            .await?;

        self.update_monitor_tags(id, &tags).await?;

        monitor.common_mut().tags = tags;

        Ok(monitor)
    }

    pub async fn connect(&self) -> Result<()> {
        *self.is_ready.lock().await = false;
        *self.socket_io.lock().await = None;

        let callback_tx = self.event_sender.clone();
        let mut builder = ClientBuilder::new(self.config.kuma.url.clone());

        for (key, value) in self
            .config
            .kuma
            .headers
            .iter()
            .filter_map(|header| header.split_once("="))
        {
            builder = builder.opening_header(key, value);
        }

        let client = builder
            .on_any(move |event, payload, _| {
                let callback_tx = callback_tx.clone();
                async move {
                    match (event, payload) {
                        (SocketIOEvent::Message, Payload::Text(params)) => {
                            if let Ok(e) = Event::from_str(
                                &params[0]
                                    .as_str()
                                    .log_warn(|| "Error while deserializing Event...")
                                    .unwrap_or(""),
                            ) {
                                callback_tx
                                    .send((e, json!(null)))
                                    .await
                                    .log_warn(|_| "Error while sending Message event")
                                    .unwrap();
                            }
                        }
                        (event, Payload::Text(params)) => {
                            if let Ok(e) = Event::from_str(&String::from(event)) {
                                callback_tx
                                    .send((e, params.into_iter().next().unwrap()))
                                    .await
                                    .log_warn(|_| "Error while sending event")
                                    .unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                .boxed()
            })
            .connect()
            .await
            .log_error(|_| "Error during connect")
            .ok();

        self.event_sender
            .send((Event::Connect, json!(null)))
            .await
            .ok();

        *self.socket_io.lock().await = client;

        for i in 0..10 {
            if *self.is_ready.lock().await {
                debug!("Connected!");
                return Ok(());
            }
            debug!("Waiting for Kuma to get ready...");
            tokio::time::sleep(Duration::from_millis(200 * i)).await;
        }

        warn!("Timeout while waiting for Kuma to get ready...");
        Err(Error::ConnectionTimeout)
    }
    pub async fn disconnect(&self) -> Result<()> {
        let mut lock = self.socket_io.lock().await;
        if let Some(socket_io) = &*lock {
            socket_io
                .disconnect()
                .await
                .map_err(|e| Error::CommunicationError(e.to_string()))?;
        }

        *lock = None;

        Ok(())
    }
}

pub struct Client {
    worker: Arc<Worker>,
}

impl Client {
    pub async fn connect(config: Arc<Config>) -> Result<Client> {
        let (tx, mut rx) = mpsc::channel::<EventArgs>(100);

        let worker = Arc::new(Worker::new(config, tx));

        let worker_ref = worker.clone();
        tokio::spawn(async move {
            while let Some((event, payload)) = rx.recv().await {
                if let Err(err) = worker_ref.on_event(event, payload).await {
                    print!("{:?}", err);
                };
            }
        });

        worker.connect().await?;

        Ok(Self { worker })
    }

    pub async fn monitors(&self) -> Result<MonitorList> {
        match *self.worker.is_ready.lock().await {
            true => Ok(self.worker.monitors.lock().await.clone()),
            false => Err(Error::NotReady),
        }
    }

    pub async fn tags(&self) -> Result<Vec<TagDefinition>> {
        match *self.worker.is_ready.lock().await {
            true => Ok(self.worker.tags.lock().await.clone()),
            false => Err(Error::NotReady),
        }
    }

    pub async fn add_tag(&self, tag: TagDefinition) -> Result<TagDefinition> {
        self.worker.add_tag(tag).await
    }

    pub async fn add_monitor(&self, monitor: Monitor) -> Result<Monitor> {
        self.worker.add_monitor(monitor).await
    }

    pub async fn edit_monitor(&self, monitor: Monitor) -> Result<Monitor> {
        self.worker.edit_monitor(monitor).await
    }

    pub async fn delete_monitor(&self, monitor_id: i32) -> Result<()> {
        self.worker.delete_monitor(monitor_id).await
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.worker.disconnect().await
    }
}
