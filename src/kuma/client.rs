use super::{Event, Monitor, MonitorList, MonitorType, Tag, TagList};
use crate::config::Config;
use crate::util::ResultLogger;
use futures_util::FutureExt;
use itertools::Itertools;
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
type Err = ();
type Result<T> = std::result::Result<T, Err>;

struct Worker {
    config: Arc<Config>,
    socket_io: Arc<Mutex<Option<SocketIO>>>,
    event_sender: Arc<Sender>,
    tags: Arc<Mutex<TagList>>,
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

    async fn on_monitor_list(&self, monitor_list: MonitorList) {
        *self.monitors.lock().await = monitor_list;

        let tags = self.get_tags().await;
        *self.tags.lock().await = tags;
        *self.is_ready.lock().await = true;
    }

    async fn on_connect(&self) {
        if let (Some(username), Some(password)) =
            (&self.config.kuma.username, &self.config.kuma.password)
        {
            self.login(username, password, self.config.kuma.mfa_token.clone())
                .await;
        }
    }

    async fn on_event(&self, event: Event, payload: Value) {
        match event {
            Event::MonitorList => {
                self.on_monitor_list(serde_json::from_value(payload).unwrap())
                    .await
            }
            Event::Connect => self.on_connect().await,
            _ => {}
        }
    }

    fn verify_response<T: DeserializeOwned>(
        response: Vec<Value>,
        result_ptr: impl AsRef<str>,
    ) -> Result<T> {
        json!(response)
            .pointer(&format!("/0/0{}", result_ptr.as_ref()))
            .and_then(|value| serde_json::from_value(value.to_owned()).ok())
            .ok_or_else(|| ())
    }

    async fn get_tags(&self) -> TagList {
        self.call("getTags", vec![], "/tags", Duration::from_secs(2))
            .await
            .unwrap_or_default()
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

        for i in 0..5 {
            let method = method.clone();
            let args = args.clone();
            let result_ptr = result_ptr.clone();
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<T>>(1);

            if let Some(socket_io) = &*self.socket_io.lock().await {
                let result = socket_io
                    .emit_with_ack(
                        method,
                        Payload::Text(args.into_iter().collect_vec()),
                        timeout,
                        move |message: Payload, _: SocketIO| {
                            let tx = tx.clone();
                            let result_ptr = result_ptr.clone();
                            async move {
                                match message {
                                    Payload::Text(response) => {
                                        let _ = tx
                                            .send(Self::verify_response(response, result_ptr))
                                            .await;
                                    }
                                    _ => {}
                                }
                            }
                            .boxed()
                        },
                    )
                    .await;

                match result {
                    Ok(_) => match tokio::time::timeout(Duration::from_secs(3), rx.recv()).await {
                        Ok(Some(Ok(value))) => return Ok(value),
                        _ => {
                            println!("Error during send");
                        }
                    },
                    Err(e) => {
                        println!("{}", e.to_string());
                    }
                };
            }

            tokio::time::sleep(Duration::from_millis(200 * i)).await;

            println!("Reconnecting...");
            self.connect().await;
        }

        Err(())
    }

    pub async fn login(
        &self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        token: Option<String>,
    ) -> bool {
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
        .unwrap_or_else(|_| false)
    }

    pub async fn add_tag(&self, tag: Tag) -> Tag {
        self.call(
            "addTag",
            vec![serde_json::to_value(tag.clone()).unwrap()],
            "/tag",
            Duration::from_secs(2),
        )
        .await
        .unwrap_or_else(|_| tag)
    }

    pub async fn add_monitor_tag(&self, monitor_id: i32, tag_id: i32, value: Option<String>) {
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
            .await
            .unwrap_or_default();
    }

    pub async fn delete_monitor(&self, monitor_id: i32) {
        let _: bool = self
            .call(
                "deleteMonitor",
                vec![json!(monitor_id)],
                "/ok",
                Duration::from_secs(2),
            )
            .await
            .unwrap_or_default();
    }

    async fn resolve_group(&self, monitor: &mut Monitor) -> bool {
        if let Some(group_name) = &monitor.common().group {
            if let Some(Some(group_id)) = self
                .monitors
                .lock()
                .await
                .iter()
                .find(|x| {
                    x.1.monitor_type() == MonitorType::Group
                        && x.1.common().tags.iter().any(|tag| {
                            tag.name == "AutoKuma"
                                && tag
                                    .value
                                    .as_ref()
                                    .is_some_and(|tag_value| tag_value == group_name)
                        })
                })
                .map(|x| x.1.common().id)
            {
                monitor.common_mut().parent = Some(group_id);
            } else {
                return false;
            }
        }
        return true;
    }

    async fn update_monitor_tags(&self, monitor_id: i32, tags: &Vec<Tag>) {
        for tag in tags {
            if let Some(tag_id) = tag.id {
                self.add_monitor_tag(monitor_id, tag_id, tag.value.clone())
                    .await;
            }
        }
    }

    pub async fn add_monitor(&self, mut monitor: Monitor) -> Monitor {
        let mut tags = vec![];
        mem::swap(&mut tags, &mut monitor.common_mut().tags);

        if !self.resolve_group(&mut monitor).await {
            return monitor;
        }

        let id: i32 = match self
            .call(
                "add",
                vec![serde_json::to_value(&monitor).unwrap()],
                "/monitorID",
                Duration::from_secs(2),
            )
            .await
        {
            Ok(id) => id,
            Err(_) => return monitor,
        };

        self.update_monitor_tags(id, &tags).await;

        monitor.common_mut().id = Some(id);
        monitor.common_mut().tags = tags;

        self.monitors
            .lock()
            .await
            .insert(id.to_string(), monitor.clone());

        monitor
    }

    pub async fn edit_monitor(&self, mut monitor: Monitor) -> Monitor {
        if !self.resolve_group(&mut monitor).await {
            return monitor;
        }

        // TODO: Tags
        let mut tags = vec![];
        mem::swap(&mut tags, &mut monitor.common_mut().tags);

        let _: i32 = self
            .call(
                "editMonitor",
                vec![serde_json::to_value(&monitor).unwrap()],
                "/monitorID",
                Duration::from_secs(2),
            )
            .await
            .unwrap_or_default();

        monitor.common_mut().tags = tags;

        monitor
    }

    pub async fn connect(&self) {
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
                                    .on_error_log(|| "Error while deserializing Event...")
                                    .unwrap_or(""),
                            ) {
                                callback_tx
                                    .send((e, json!(null)))
                                    .await
                                    .on_error_log(|_| "Error while sending Message event")
                                    .unwrap();
                            }
                        }
                        (event, Payload::Text(params)) => {
                            if let Ok(e) = Event::from_str(&String::from(event)) {
                                callback_tx
                                    .send((e, params.into_iter().next().unwrap()))
                                    .await
                                    .on_error_log(|_| "Error while sending event")
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
            .on_error_log(|_| "Error during connect")
            .ok();

        self.event_sender
            .send((Event::Connect, json!(null)))
            .await
            .ok();

        *self.socket_io.lock().await = client;

        for i in 0..10 {
            if *self.is_ready.lock().await {
                println!("Connected!");
                return;
            }
            println!("Waiting for Kuma to get ready...");
            tokio::time::sleep(Duration::from_millis(200 * i)).await;
        }

        println!("Timeout while waiting for Kuma to get ready...");
    }
}

pub struct Client {
    worker: Arc<Worker>,
}

impl Client {
    pub async fn connect(config: Arc<Config>) -> Client {
        let (tx, mut rx) = mpsc::channel::<EventArgs>(100);

        let worker = Arc::new(Worker::new(config, tx));

        let worker_ref = worker.clone();
        tokio::spawn(async move {
            while let Some((event, payload)) = rx.recv().await {
                worker_ref.on_event(event, payload).await;
            }
        });

        worker.connect().await;

        Self { worker }
    }

    pub async fn monitors(&self) -> MonitorList {
        self.worker.monitors.lock().await.clone()
    }

    pub async fn tags(&self) -> TagList {
        self.worker.tags.lock().await.clone()
    }

    pub async fn add_tag(&self, tag: Tag) -> Tag {
        self.worker.add_tag(tag).await
    }

    pub async fn add_monitor(&self, monitor: Monitor) -> Monitor {
        self.worker.add_monitor(monitor).await
    }

    pub async fn edit_monitor(&self, monitor: Monitor) -> Monitor {
        self.worker.edit_monitor(monitor).await
    }

    pub async fn delete_monitor(&self, monitor_id: i32) {
        self.worker.delete_monitor(monitor_id).await
    }
}
