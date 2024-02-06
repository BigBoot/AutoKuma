use crate::{
    error::{Error, Result},
    event::Event,
    maintenance::{Maintenance, MaintenanceList, MaintenanceMonitor, MaintenanceStatusPage},
    monitor::{Monitor, MonitorList, MonitorType},
    notification::{Notification, NotificationList},
    response::LoginResponse,
    status_page::{PublicGroupList, StatusPage, StatusPageList},
    tag::{Tag, TagDefinition},
    util::ResultLogger,
    Config,
};
use futures_util::FutureExt;
use itertools::Itertools;
use log::{debug, trace, warn};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rust_socketio::{
    asynchronous::{Client as SocketIO, ClientBuilder},
    Event as SocketIOEvent, Payload,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    mem,
    str::FromStr,
    sync::{Arc, Weak},
    time::Duration,
};
use tokio::{runtime::Handle, sync::Mutex};

struct Ready {
    pub monitor_list: bool,
    pub notification_list: bool,
    pub maintenance_list: bool,
    pub status_page_list: bool,
}

impl Ready {
    pub fn new() -> Self {
        Self {
            monitor_list: false,
            notification_list: false,
            maintenance_list: false,
            status_page_list: false,
        }
    }

    pub fn reset(&mut self) {
        *self = Ready::new()
    }

    pub fn is_ready(&self) -> bool {
        self.monitor_list
            && self.notification_list
            && self.maintenance_list
            && self.status_page_list
    }
}

struct Worker {
    config: Arc<Config>,
    socket_io: Arc<Mutex<Option<SocketIO>>>,
    monitors: Arc<Mutex<MonitorList>>,
    notifications: Arc<Mutex<NotificationList>>,
    maintenances: Arc<Mutex<MaintenanceList>>,
    status_pages: Arc<Mutex<StatusPageList>>,
    is_connected: Arc<Mutex<bool>>,
    is_ready: Arc<Mutex<Ready>>,
    is_logged_in: Arc<Mutex<bool>>,
    reqwest: Arc<Mutex<reqwest::Client>>,
}

impl Worker {
    fn new(config: Config) -> Arc<Self> {
        Arc::new(Worker {
            config: Arc::new(config.clone()),
            socket_io: Arc::new(Mutex::new(None)),
            monitors: Default::default(),
            notifications: Default::default(),
            maintenances: Default::default(),
            status_pages: Default::default(),
            is_connected: Arc::new(Mutex::new(false)),
            is_ready: Arc::new(Mutex::new(Ready::new())),
            is_logged_in: Arc::new(Mutex::new(false)),
            reqwest: Arc::new(Mutex::new(
                reqwest::Client::builder()
                    .default_headers(HeaderMap::from_iter(
                        config
                            .headers
                            .iter()
                            .filter_map(|header| header.split_once("="))
                            .filter_map(|(key, value)| {
                                match (
                                    HeaderName::from_bytes(key.as_bytes()),
                                    HeaderValue::from_bytes(value.as_bytes()),
                                ) {
                                    (Ok(key), Ok(value)) => Some((key, value)),
                                    _ => None,
                                }
                            }),
                    ))
                    .build()
                    .unwrap(),
            )),
        })
    }

    async fn on_monitor_list(self: &Arc<Self>, monitor_list: MonitorList) -> Result<()> {
        *self.monitors.lock().await = monitor_list;
        self.is_ready.lock().await.monitor_list = true;

        Ok(())
    }

    async fn on_notification_list(
        self: &Arc<Self>,
        notification_list: NotificationList,
    ) -> Result<()> {
        *self.notifications.lock().await = notification_list;
        self.is_ready.lock().await.notification_list = true;

        Ok(())
    }

    async fn on_maintenance_list(
        self: &Arc<Self>,
        maintenance_list: MaintenanceList,
    ) -> Result<()> {
        *self.maintenances.lock().await = maintenance_list;
        self.is_ready.lock().await.maintenance_list = true;

        Ok(())
    }

    async fn on_status_page_list(self: &Arc<Self>, status_page_list: StatusPageList) -> Result<()> {
        *self.status_pages.lock().await = status_page_list;
        self.is_ready.lock().await.status_page_list = true;

        Ok(())
    }

    async fn on_info(self: &Arc<Self>) -> Result<()> {
        *self.is_connected.lock().await = true;
        if let (Some(username), Some(password), true) = (
            &self.config.username,
            &self.config.password,
            !*self.is_logged_in.lock().await,
        ) {
            self.login(username, password, self.config.mfa_token.clone())
                .await?;
        }

        Ok(())
    }

    async fn on_auto_login(self: &Arc<Self>) -> Result<()> {
        debug!("Logged in using AutoLogin!");
        *self.is_logged_in.lock().await = true;
        Ok(())
    }

    async fn on_event(self: &Arc<Self>, event: Event, payload: Value) -> Result<()> {
        match event {
            Event::MonitorList => {
                self.on_monitor_list(serde_json::from_value(payload).unwrap())
                    .await?
            }
            Event::NotificationList => {
                self.on_notification_list(serde_json::from_value(payload).unwrap())
                    .await?
            }
            Event::MaintenanceList => {
                self.on_maintenance_list(serde_json::from_value(payload).unwrap())
                    .await?
            }
            Event::StatusPageList => {
                self.on_status_page_list(serde_json::from_value(payload).unwrap())
                    .await?
            }
            Event::Info => self.on_info().await?,
            Event::AutoLogin => self.on_auto_login().await?,
            _ => {}
        }

        Ok(())
    }

    fn extract_response<T: DeserializeOwned>(
        response: Vec<Value>,
        result_ptr: impl AsRef<str>,
        verify: bool,
    ) -> Result<T> {
        let json = json!(response);

        if verify
            && !json
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

    async fn call<A, T>(
        self: &Arc<Self>,
        method: impl Into<String>,
        args: A,
        result_ptr: impl Into<String>,
        verify: bool,
    ) -> Result<T>
    where
        A: IntoIterator<Item = Value> + Send + Clone,
        T: DeserializeOwned + Send + 'static,
    {
        let method = method.into();
        let result_ptr: String = result_ptr.into();

        let method_ref = method.clone();
        let args: A = args.clone();
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
                Duration::from_secs_f64(self.config.call_timeout),
                move |message: Payload, _: SocketIO| {
                    debug!("call {} -> {:?}", method_ref, &message);
                    let tx = tx.clone();
                    let result_ptr = result_ptr.clone();
                    async move {
                        _ = match message {
                            Payload::Text(response) => {
                                tx.send(Self::extract_response(response, result_ptr, verify))
                                    .await
                            }
                            _ => tx.send(Err(Error::UnsupportedResponse)).await,
                        }
                    }
                    .boxed()
                },
            )
            .await
            .map_err(|e| Error::CommunicationError(e.to_string()))?;

        let result =
            tokio::time::timeout(Duration::from_secs_f64(self.config.call_timeout), rx.recv())
                .await
                .map_err(|_| Error::CallTimeout(method.clone()))?
                .ok_or_else(|| Error::CallTimeout(method))?;

        result
    }

    pub async fn login(
        self: &Arc<Self>,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        token: Option<String>,
    ) -> Result<()> {
        let result: Result<LoginResponse> = self
            .call(
                "login",
                vec![serde_json::to_value(HashMap::from([
                    ("username", json!(username.as_ref())),
                    ("password", json!(password.as_ref())),
                    ("token", json!(token)),
                ]))
                .unwrap()],
                "",
                false,
            )
            .await;

        match result {
            Ok(LoginResponse { ok: true, .. }) => {
                debug!("Logged in as {}!", username.as_ref());
                *self.is_logged_in.lock().await = true;
                Ok(())
            }
            Ok(LoginResponse {
                ok: false,
                msg: Some(msg),
                ..
            }) => Err(Error::LoginError(msg)),
            Err(e) => {
                *self.is_logged_in.lock().await = false;
                Err(e)
            }
            _ => {
                *self.is_logged_in.lock().await = false;
                Err(Error::LoginError("Unexpect login response".to_owned()))
            }
        }
        .log_warn(|e| e.to_string())
    }

    async fn get_tags(self: &Arc<Self>) -> Result<Vec<TagDefinition>> {
        self.call("getTags", vec![], "/tags", true).await
    }

    pub async fn add_tag(self: &Arc<Self>, tag: &mut TagDefinition) -> Result<()> {
        *tag = self
            .call(
                "addTag",
                vec![serde_json::to_value(tag.clone()).unwrap()],
                "/tag",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn edit_tag(self: &Arc<Self>, tag: &mut TagDefinition) -> Result<()> {
        *tag = self
            .call(
                "editTag",
                vec![serde_json::to_value(tag.clone()).unwrap()],
                "/tag",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn delete_tag(self: &Arc<Self>, tag_id: i32) -> Result<()> {
        let _: bool = self
            .call("deleteTag", vec![json!(tag_id)], "/ok", true)
            .await?;

        Ok(())
    }

    pub async fn add_notification(self: &Arc<Self>, notification: &mut Notification) -> Result<()> {
        self.edit_notification(notification).await
    }

    pub async fn edit_notification(
        self: &Arc<Self>,
        notification: &mut Notification,
    ) -> Result<()> {
        let json = serde_json::to_value(notification.clone()).unwrap();
        let config_json = serde_json::to_value(notification.config.clone()).unwrap();

        let merge = serde_merge::omerge(config_json, &json).unwrap();

        notification.id = Some(
            self.call(
                "addNotification",
                vec![merge, notification.id.into()],
                "/id",
                true,
            )
            .await?,
        );

        Ok(())
    }

    pub async fn delete_notification(self: &Arc<Self>, notification_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "deleteNotification",
                vec![json!(notification_id)],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn add_monitor_tag(
        self: &Arc<Self>,
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
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn edit_monitor_tag(
        self: &Arc<Self>,
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
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn delete_monitor_tag(
        self: &Arc<Self>,
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
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn delete_monitor(self: &Arc<Self>, monitor_id: i32) -> Result<()> {
        let _: bool = self
            .call("deleteMonitor", vec![json!(monitor_id)], "/ok", true)
            .await?;

        Ok(())
    }

    async fn resolve_group(self: &Arc<Self>, monitor: &mut Monitor) -> Result<()> {
        if let Some(group_name) = monitor.common().parent_name().clone() {
            if let Some(Some(group_id)) = self
                .monitors
                .lock()
                .await
                .iter()
                .find(|x| {
                    x.1.monitor_type() == MonitorType::Group
                        && x.1.common().tags().iter().any(|tag| {
                            tag.name.as_ref().is_some_and(|tag| tag == "AutoKuma")
                                && tag
                                    .value
                                    .as_ref()
                                    .is_some_and(|tag_value| tag_value == &group_name)
                        })
                })
                .map(|x| *x.1.common().id())
            {
                *monitor.common_mut().parent_mut() = Some(group_id);
            } else {
                return Err(Error::GroupNotFound(group_name));
            }
        } else {
            *monitor.common_mut().parent_mut() = None;
        }
        return Ok(());
    }

    async fn update_monitor_tags(self: &Arc<Self>, monitor_id: i32, tags: &Vec<Tag>) -> Result<()> {
        let new_tags = tags
            .iter()
            .filter_map(|tag| tag.tag_id.and_then(|id| Some((id, tag))))
            .collect::<HashMap<_, _>>();

        if let Some(monitor) = self.monitors.lock().await.get(&monitor_id.to_string()) {
            let current_tags = monitor
                .common()
                .tags()
                .iter()
                .filter_map(|tag| tag.tag_id.and_then(|id| Some((id, tag))))
                .collect::<HashMap<_, _>>();

            let duplicates = monitor
                .common()
                .tags()
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

    pub async fn add_monitor(self: &Arc<Self>, monitor: &mut Monitor) -> Result<()> {
        self.resolve_group(monitor).await?;

        let tags = mem::take(monitor.common_mut().tags_mut());
        let notifications = mem::take(monitor.common_mut().notification_id_list_mut());

        let id: i32 = self
            .clone()
            .call(
                "add",
                vec![serde_json::to_value(&monitor).unwrap()],
                "/monitorID",
                true,
            )
            .await?;

        *monitor.common_mut().id_mut() = Some(id);
        *monitor.common_mut().notification_id_list_mut() = notifications;
        *monitor.common_mut().tags_mut() = tags;

        self.edit_monitor(monitor).await?;

        self.monitors
            .lock()
            .await
            .insert(id.to_string(), monitor.clone());

        Ok(())
    }

    pub async fn get_monitor(self: &Arc<Self>, monitor_id: i32) -> Result<Monitor> {
        self.call(
            "getMonitor",
            vec![serde_json::to_value(monitor_id.clone()).unwrap()],
            "/monitor",
            true,
        )
        .await
        .map_err(|e| match e {
            Error::ServerError(msg) if msg.contains("Cannot read properties of null") => {
                Error::IdNotFound("Monitor".to_owned(), monitor_id)
            }
            _ => e,
        })
    }

    pub async fn edit_monitor(self: &Arc<Self>, monitor: &mut Monitor) -> Result<()> {
        self.resolve_group(monitor).await?;

        let tags = mem::take(monitor.common_mut().tags_mut());

        let id: i32 = self
            .call(
                "editMonitor",
                vec![serde_json::to_value(&monitor).unwrap()],
                "/monitorID",
                true,
            )
            .await?;

        self.update_monitor_tags(id, &tags).await?;

        *monitor.common_mut().tags_mut() = tags;

        Ok(())
    }

    pub async fn pause_monitor(self: &Arc<Self>, monitor_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "pauseMonitor",
                vec![serde_json::to_value(monitor_id).unwrap()],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn resume_monitor(self: &Arc<Self>, monitor_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "resumeMonitor",
                vec![serde_json::to_value(monitor_id).unwrap()],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    async fn get_maintenance_monitors(
        self: &Arc<Self>,
        maintenance_id: i32,
    ) -> Result<Vec<MaintenanceMonitor>> {
        self.call(
            "getMonitorMaintenance",
            vec![serde_json::to_value(maintenance_id).unwrap()],
            "/monitors",
            true,
        )
        .await
    }

    async fn set_maintenance_monitors(
        self: &Arc<Self>,
        maintenance_id: i32,
        monitors: &Vec<MaintenanceMonitor>,
    ) -> Result<()> {
        let _: bool = self
            .call(
                "addMonitorMaintenance",
                vec![
                    serde_json::to_value(maintenance_id).unwrap(),
                    serde_json::to_value(monitors).unwrap(),
                ],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    async fn get_maintenance_status_pages(
        self: &Arc<Self>,
        maintenance_id: i32,
    ) -> Result<Vec<MaintenanceStatusPage>> {
        self.call(
            "getMaintenanceStatusPage",
            vec![serde_json::to_value(maintenance_id).unwrap()],
            "/statusPages",
            true,
        )
        .await
    }

    async fn set_maintenance_status_pages(
        self: &Arc<Self>,
        maintenance_id: i32,
        status_pages: &Vec<MaintenanceStatusPage>,
    ) -> Result<()> {
        let _: bool = self
            .call(
                "addMaintenanceStatusPage",
                vec![
                    serde_json::to_value(maintenance_id).unwrap(),
                    serde_json::to_value(status_pages).unwrap(),
                ],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn delete_maintenance(self: &Arc<Self>, maintenance_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "deleteMaintenance",
                vec![json!(maintenance_id)],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn add_maintenance(self: &Arc<Self>, maintenance: &mut Maintenance) -> Result<()> {
        let id = self
            .call(
                "addMaintenance",
                vec![serde_json::to_value(maintenance.clone()).unwrap()],
                "/maintenanceID",
                true,
            )
            .await?;

        maintenance.common_mut().id = Some(id);
        if let Some(monitors) = &maintenance.common().monitors {
            self.set_maintenance_monitors(id, monitors).await?;
        }
        if let Some(status_pages) = &maintenance.common().status_pages {
            self.set_maintenance_status_pages(id, status_pages).await?;
        }

        Ok(())
    }

    pub async fn get_maintenance(self: &Arc<Self>, maintenance_id: i32) -> Result<Maintenance> {
        let mut maintenance: Maintenance = self
            .call(
                "getMaintenance",
                vec![serde_json::to_value(maintenance_id.clone()).unwrap()],
                "/maintenance",
                true,
            )
            .await
            .map_err(|e| match e {
                Error::ServerError(msg) if msg.contains("Cannot read properties of null") => {
                    Error::IdNotFound("Maintenance".to_owned(), maintenance_id)
                }
                _ => e,
            })?;

        maintenance.common_mut().monitors =
            Some(self.get_maintenance_monitors(maintenance_id).await?);
        maintenance.common_mut().status_pages =
            Some(self.get_maintenance_status_pages(maintenance_id).await?);

        Ok(maintenance)
    }

    pub async fn edit_maintenance(self: &Arc<Self>, maintenance: &mut Maintenance) -> Result<()> {
        let id = self
            .call(
                "addMaintenance",
                vec![serde_json::to_value(maintenance.clone()).unwrap()],
                "/maintenanceID",
                true,
            )
            .await?;

        maintenance.common_mut().id = Some(id);
        if let Some(monitors) = &maintenance.common().monitors {
            self.set_maintenance_monitors(id, monitors).await?;
        }
        if let Some(status_pages) = &maintenance.common().status_pages {
            self.set_maintenance_status_pages(id, status_pages).await?;
        }

        Ok(())
    }

    pub async fn pause_maintenance(self: &Arc<Self>, maintenance_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "pauseMaintenance",
                vec![serde_json::to_value(maintenance_id).unwrap()],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn resume_maintenance(self: &Arc<Self>, maintenance_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "resumeMaintenance",
                vec![serde_json::to_value(maintenance_id).unwrap()],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    async fn get_public_group_list(self: &Arc<Self>, slug: &str) -> Result<PublicGroupList> {
        let response: Value = self
            .reqwest
            .lock()
            .await
            .get(
                self.config
                    .url
                    .join(&format!("/api/status-page/{}", slug))
                    .map_err(|e| Error::InvalidUrl(e.to_string()))?,
            )
            .send()
            .await?
            .json()
            .await?;

        let monitor_list = response
            .clone()
            .pointer("/publicGroupList")
            .ok_or_else(|| {
                Error::InvalidResponse(vec![response.clone()], "/publicGroupList".to_owned())
            })?
            .clone();

        Ok(serde_json::from_value(monitor_list)
            .log_warn(|e| e.to_string())
            .map_err(|_| Error::UnsupportedResponse)?)
    }

    pub async fn delete_status_page(self: &Arc<Self>, slug: &str) -> Result<()> {
        let _: bool = self
            .call("deleteStatusPage", vec![json!(slug)], "/ok", true)
            .await?;

        Ok(())
    }

    pub async fn add_status_page(self: &Arc<Self>, status_page: &mut StatusPage) -> Result<()> {
        let ok: bool = self
            .call(
                "addStatusPage",
                vec![
                    serde_json::to_value(status_page.title.clone()).unwrap(),
                    serde_json::to_value(status_page.slug.clone()).unwrap(),
                ],
                "/ok",
                true,
            )
            .await?;

        if !ok {
            return Err(Error::ServerError("Unable to add status page".to_owned()));
        }

        self.edit_status_page(status_page).await?;

        Ok(())
    }

    pub async fn get_status_page(self: &Arc<Self>, slug: &str) -> Result<StatusPage> {
        let mut status_page: StatusPage = self
            .call(
                "getStatusPage",
                vec![serde_json::to_value(slug).unwrap()],
                "/config",
                true,
            )
            .await
            .map_err(|e| match e {
                Error::ServerError(msg) if msg.contains("Cannot read properties of null") => {
                    Error::SlugNotFound("StatusPage".to_owned(), slug.to_owned())
                }
                _ => e,
            })?;

        status_page.public_group_list = Some(
            self.get_public_group_list(&status_page.slug.clone().unwrap_or_default())
                .await?,
        );

        Ok(status_page)
    }

    pub async fn edit_status_page(self: &Arc<Self>, status_page: &mut StatusPage) -> Result<()> {
        let _: bool = self
            .call(
                "saveStatusPage",
                vec![
                    serde_json::to_value(status_page.slug.clone()).unwrap(),
                    serde_json::to_value(status_page.clone()).unwrap(),
                    serde_json::to_value(status_page.icon.clone()).unwrap(),
                    serde_json::to_value(status_page.public_group_list.clone()).unwrap(),
                ],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn connect(self: &Arc<Self>) -> Result<()> {
        self.is_ready.lock().await.reset();
        *self.is_logged_in.lock().await = false;
        *self.socket_io.lock().await = None;

        let mut builder = ClientBuilder::new(
            self.config
                .url
                .join("/socket.io/")
                .map_err(|e| Error::InvalidUrl(e.to_string()))?,
        )
        .transport_type(rust_socketio::TransportType::Websocket);

        for (key, value) in self
            .config
            .headers
            .iter()
            .filter_map(|header| header.split_once("="))
        {
            builder = builder.opening_header(key, value);
        }

        let handle = Handle::current();
        let self_ref = Arc::downgrade(self);
        let client = builder
            .on_any(move |event, payload, _| {
                let handle = handle.clone();
                let self_ref: Weak<Worker> = self_ref.clone();
                trace!("Client::on_any({:?}, {:?})", &event, &payload);
                async move {
                    if let Some(arc) = self_ref.upgrade() {
                        match (event, payload) {
                            (SocketIOEvent::Message, Payload::Text(params)) => {
                                if let Ok(e) = Event::from_str(
                                    &params[0]
                                        .as_str()
                                        .log_warn(|| "Error while deserializing Event...")
                                        .unwrap_or(""),
                                ) {
                                    handle.clone().spawn(async move {
                                        _ = arc.on_event(e, json!(null)).await.log_warn(|e| {
                                            format!(
                                                "Error while sending message event: {}",
                                                e.to_string()
                                            )
                                        });
                                    });
                                }
                            }
                            (event, Payload::Text(params)) => {
                                if let Ok(e) = Event::from_str(&String::from(event)) {
                                    handle.clone().spawn(async move {
                                        _ = arc
                                            .on_event(e, params.into_iter().next().unwrap())
                                            .await
                                            .log_warn(|e| {
                                                format!(
                                                    "Error while sending event: {}",
                                                    e.to_string()
                                                )
                                            });
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                .boxed()
            })
            .connect()
            .await
            .log_error(|_| "Error during connect")
            .ok();

        debug!("Waiting for connection");

        debug!("Connection opened!");
        *self.socket_io.lock().await = client;

        for i in 0..10 {
            if self.is_ready().await {
                debug!("Connected!");
                return Ok(());
            }

            debug!("Waiting for Kuma to get ready...");
            tokio::time::sleep(Duration::from_millis(200 * i)).await;
        }

        warn!("Timeout while waiting for Kuma to get ready...");
        match *self.is_connected.lock().await {
            true => Err(Error::NotAuthenticated),
            false => Err(Error::ConnectionTimeout),
        }
    }

    pub async fn disconnect(self: &Arc<Self>) -> Result<()> {
        let self_ref = self.to_owned();
        tokio::spawn(async move {
            let socket_io = self_ref.socket_io.lock().await;
            if let Some(socket_io) = &*socket_io {
                _ = socket_io.disconnect().await;
            }
            drop(socket_io);
            *self_ref.socket_io.lock().await = None;
            debug!("Connection closed!");
        });

        Ok(())
    }

    pub async fn is_ready(self: &Arc<Self>) -> bool {
        self.is_ready.lock().await.is_ready()
    }
}

/// A client for interacting with Uptime Kuma.
pub struct Client {
    worker: Arc<Worker>,
}

impl Client {
    /// Establishes a connection to Uptime Kuma with the provided configuration.
    pub async fn connect(config: Config) -> Result<Client> {
        let worker = Worker::new(config);
        worker.connect().await?;

        Ok(Self { worker })
    }

    /// Retrieves a list of monitors from Uptime Kuma.
    pub async fn get_monitors(&self) -> Result<MonitorList> {
        match self.worker.is_ready().await {
            true => Ok(self.worker.monitors.lock().await.clone()),
            false => Err(Error::NotReady),
        }
    }

    /// Retrieves information about a specific monitor identified by its ID.
    pub async fn get_monitor(&self, monitor_id: i32) -> Result<Monitor> {
        self.worker.get_monitor(monitor_id).await
    }

    /// Adds a new monitor to Uptime Kuma.
    pub async fn add_monitor(&self, mut monitor: Monitor) -> Result<Monitor> {
        self.worker.add_monitor(&mut monitor).await?;
        Ok(monitor)
    }

    /// Edits an existing monitor in Uptime Kuma.
    pub async fn edit_monitor(&self, mut monitor: Monitor) -> Result<Monitor> {
        self.worker.edit_monitor(&mut monitor).await?;
        Ok(monitor)
    }

    /// Deletes a monitor from Uptime Kuma based on its ID.
    pub async fn delete_monitor(&self, monitor_id: i32) -> Result<()> {
        self.worker.delete_monitor(monitor_id).await
    }

    /// Pauses a monitor in Uptime Kuma based on its ID.
    pub async fn pause_monitor(&self, monitor_id: i32) -> Result<()> {
        self.worker.pause_monitor(monitor_id).await
    }

    /// Resumes a paused monitor in Uptime Kuma based on its ID.
    pub async fn resume_monitor(&self, monitor_id: i32) -> Result<()> {
        self.worker.resume_monitor(monitor_id).await
    }

    /// Retrieves a list of tags from Uptime Kuma.
    pub async fn get_tags(&self) -> Result<Vec<TagDefinition>> {
        self.worker.get_tags().await
    }

    /// Retrieves information about a specific tag identified by its ID.
    pub async fn get_tag(&self, tag_id: i32) -> Result<TagDefinition> {
        self.worker.get_tags().await.and_then(|tags| {
            tags.into_iter()
                .find(|tag| tag.tag_id == Some(tag_id))
                .ok_or_else(|| Error::IdNotFound("Tag".to_owned(), tag_id))
        })
    }

    /// Adds a new tag to Uptime Kuma.
    pub async fn add_tag(&self, mut tag: TagDefinition) -> Result<TagDefinition> {
        self.worker.add_tag(&mut tag).await?;
        Ok(tag)
    }

    /// Edits an existing tag in Uptime Kuma.
    pub async fn edit_tag(&self, mut tag: TagDefinition) -> Result<TagDefinition> {
        self.worker.edit_tag(&mut tag).await?;
        Ok(tag)
    }

    /// Deletes a tag from Uptime Kuma based on its ID.
    pub async fn delete_tag(&self, tag_id: i32) -> Result<()> {
        self.worker.delete_tag(tag_id).await
    }

    /// Retrieves a list of notifications from Uptime Kuma.
    pub async fn get_notifications(&self) -> Result<NotificationList> {
        match self.worker.is_ready().await {
            true => Ok(self.worker.notifications.lock().await.clone()),
            false => Err(Error::NotReady),
        }
    }

    /// Retrieves information about a specific notification identified by its ID.
    pub async fn get_notification(&self, notification_id: i32) -> Result<Notification> {
        self.get_notifications().await.and_then(|notifications| {
            notifications
                .into_iter()
                .find(|notification| notification.id == Some(notification_id))
                .ok_or_else(|| Error::IdNotFound("Notification".to_owned(), notification_id))
        })
    }

    /// Adds a new notification to Uptime Kuma.
    pub async fn add_notification(&self, mut notification: Notification) -> Result<Notification> {
        self.worker.add_notification(&mut notification).await?;
        Ok(notification)
    }

    /// Edits an existing notification in Uptime Kuma.
    pub async fn edit_notification(&self, mut notification: Notification) -> Result<Notification> {
        self.worker.edit_notification(&mut notification).await?;
        Ok(notification)
    }

    /// Deletes a notification from Uptime Kuma based on its ID.
    pub async fn delete_notification(&self, notification_id: i32) -> Result<()> {
        self.worker.delete_notification(notification_id).await
    }

    /// Retrieves a list of maintenances from Uptime Kuma.
    pub async fn get_maintenances(&self) -> Result<MaintenanceList> {
        match self.worker.is_ready().await {
            true => Ok(self.worker.maintenances.lock().await.clone()),
            false => Err(Error::NotReady),
        }
    }

    /// Retrieves information about a specific maintenance identified by its ID.
    pub async fn get_maintenance(&self, maintenance_id: i32) -> Result<Maintenance> {
        self.worker.get_maintenance(maintenance_id).await
    }

    /// Adds a new maintenance to Uptime Kuma.
    pub async fn add_maintenance(&self, mut maintenance: Maintenance) -> Result<Maintenance> {
        self.worker.add_maintenance(&mut maintenance).await?;
        Ok(maintenance)
    }

    /// Edits an existing maintenance in Uptime Kuma.
    pub async fn edit_maintenance(&self, mut maintenance: Maintenance) -> Result<Maintenance> {
        self.worker.edit_maintenance(&mut maintenance).await?;
        Ok(maintenance)
    }

    /// Deletes a maintenance from Uptime Kuma based on its ID.
    pub async fn delete_maintenance(&self, maintenance_id: i32) -> Result<()> {
        self.worker.delete_maintenance(maintenance_id).await
    }

    /// Pauses a maintenance in Uptime Kuma based on its ID.
    pub async fn pause_maintenance(&self, maintenance_id: i32) -> Result<()> {
        self.worker.pause_maintenance(maintenance_id).await
    }

    /// Resumes a paused maintenance in Uptime Kuma based on its ID.
    pub async fn resume_maintenance(&self, maintenance_id: i32) -> Result<()> {
        self.worker.resume_maintenance(maintenance_id).await
    }

    /// Retrieves a list of status pages from Uptime Kuma.
    pub async fn get_status_pages(&self) -> Result<StatusPageList> {
        match self.worker.is_ready().await {
            true => Ok(self.worker.status_pages.lock().await.clone()),
            false => Err(Error::NotReady),
        }
    }

    /// Retrieves information about a specific status page identified by its slug.
    pub async fn get_status_page(&self, slug: &str) -> Result<StatusPage> {
        self.worker.get_status_page(slug).await
    }

    /// Adds a new status page to Uptime Kuma.
    pub async fn add_status_page(&self, mut status_page: StatusPage) -> Result<StatusPage> {
        self.worker.add_status_page(&mut status_page).await?;
        Ok(status_page)
    }

    /// Edits an existing status page in Uptime Kuma.
    pub async fn edit_status_page(&self, mut status_page: StatusPage) -> Result<StatusPage> {
        self.worker.edit_status_page(&mut status_page).await?;
        Ok(status_page)
    }

    /// Deletes a status page from Uptime Kuma based on its slug.
    pub async fn delete_status_page(&self, slug: &str) -> Result<()> {
        self.worker.delete_status_page(slug).await
    }

    /// Disconnects the client from Uptime Kuma.
    pub async fn disconnect(&self) -> Result<()> {
        self.worker.disconnect().await
    }
}
