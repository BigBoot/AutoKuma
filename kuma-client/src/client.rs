use crate::{
    docker_host::{DockerHost, DockerHostList},
    error::{Error, InvalidReferenceError, Result, TotpResult},
    event::Event,
    maintenance::{Maintenance, MaintenanceList, MaintenanceMonitor, MaintenanceStatusPage},
    monitor::{Monitor, MonitorList},
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
use native_tls::{Certificate, TlsConnector};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rust_socketio::{
    asynchronous::{Client as SocketIO, ClientBuilder},
    Event as SocketIOEvent, Payload,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs, mem,
    str::FromStr,
    sync::{Arc, Weak},
    time::Duration,
};
use tap::prelude::*;
use tokio::{runtime::Handle, sync::Mutex};
use totp_rs::{Rfc6238, TOTP};

struct Ready {
    pub monitor_list: bool,
    pub notification_list: bool,
    pub maintenance_list: bool,
    pub status_page_list: bool,
    pub docker_host_list: bool,
}

impl Ready {
    pub fn new() -> Self {
        Self {
            monitor_list: false,
            notification_list: false,
            maintenance_list: false,
            status_page_list: false,
            docker_host_list: false,
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
            && self.docker_host_list
    }
}

struct Worker {
    config: Arc<Config>,
    #[allow(dead_code)]
    socket_io: Arc<Mutex<Option<SocketIO>>>,
    monitors: Arc<Mutex<MonitorList>>,
    notifications: Arc<Mutex<NotificationList>>,
    docker_hosts: Arc<Mutex<DockerHostList>>,
    maintenances: Arc<Mutex<MaintenanceList>>,
    status_pages: Arc<Mutex<StatusPageList>>,
    is_connected: Arc<Mutex<bool>>,
    is_ready: Arc<Mutex<Ready>>,
    is_logged_in: Arc<Mutex<bool>>,
    auth_token: Arc<Mutex<Option<String>>>,
    reqwest: Arc<Mutex<reqwest::Client>>,
    custom_cert: Option<(String, Certificate)>,
}

impl Worker {
    fn new(config: Config) -> Result<Arc<Self>> {
        let custom_cert = config
            .tls
            .cert
            .as_ref()
            .map(|file| -> Result<(String, Certificate)> {
                fs::read(file)
                    .map_err(|e| Error::InvalidTlsCert(file.clone(), e.to_string()))
                    .and_then(|content| {
                        Certificate::from_pem(&content)
                            .map_err(|e| Error::InvalidTlsCert(file.clone(), e.to_string()))
                    })
                    .map(|cert| (file.clone(), cert))
            })
            .transpose()?;

        let mut reqwest_builder = reqwest::Client::builder()
            .danger_accept_invalid_certs(!config.tls.verify)
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
            ));

        if let Some((file, cert)) = &custom_cert {
            reqwest_builder = reqwest_builder.add_root_certificate(
                reqwest::Certificate::from_der(
                    &cert
                        .to_der()
                        .map_err(|e| Error::InvalidTlsCert(file.clone(), e.to_string()))?,
                )
                .map_err(|e| Error::InvalidTlsCert(file.clone(), e.to_string()))?,
            );
        }

        Ok(Arc::new(Worker {
            config: Arc::new(config.clone()),
            socket_io: Arc::new(Mutex::new(None)),
            monitors: Default::default(),
            notifications: Default::default(),
            maintenances: Default::default(),
            status_pages: Default::default(),
            docker_hosts: Default::default(),
            is_connected: Arc::new(Mutex::new(false)),
            is_ready: Arc::new(Mutex::new(Ready::new())),
            is_logged_in: Arc::new(Mutex::new(false)),
            auth_token: Arc::new(Mutex::new(config.auth_token)),
            reqwest: Arc::new(Mutex::new(reqwest_builder.build().unwrap())),
            custom_cert: custom_cert,
        }))
    }

    fn get_mfa_token(self: &Arc<Self>) -> TotpResult<Option<String>> {
        Ok(match &self.config.mfa_secret {
            Some(secret) => {
                let totp = match secret {
                    url if url.starts_with("otpauth://") => TOTP::from_url(url)?,
                    secret => TOTP::from_rfc6238(Rfc6238::with_defaults(
                        totp_rs::Secret::Encoded(secret.clone()).to_bytes()?,
                    )?)?,
                };
                Some(totp.generate_current()?)
            }
            None => self.config.mfa_token.clone(),
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

    async fn on_docker_host_list(self: &Arc<Self>, docker_host_list: DockerHostList) -> Result<()> {
        *self.docker_hosts.lock().await = docker_host_list;
        self.is_ready.lock().await.docker_host_list = true;

        Ok(())
    }

    async fn on_info(self: &Arc<Self>) -> Result<()> {
        *self.is_connected.lock().await = true;
        let logged_in = *self.is_logged_in.lock().await;

        if !logged_in {
            let auth_token = self.auth_token.lock().await.clone();

            // Try logging in with a token if available
            if let Some(auth_token) = auth_token {
                if self.login_by_token(auth_token).await.is_ok() {
                    return Ok(());
                }
            }

            if let (Some(username), Some(password)) = (&self.config.username, &self.config.password)
            {
                let mfa_token = self.get_mfa_token()?;
                self.login(username, password, mfa_token).await?;
            }
        }

        Ok(())
    }

    async fn on_login_required(self: &Arc<Self>) -> Result<()> {
        Ok(())
    }

    async fn on_auto_login(self: &Arc<Self>) -> Result<()> {
        debug!("Logged in using AutoLogin!");
        *self.is_logged_in.lock().await = true;
        Ok(())
    }

    async fn on_delete_monitor_from_list(self: &Arc<Self>, monitor_id: i32) -> Result<()> {
        self.monitors.lock().await.remove(&monitor_id.to_string());
        Ok(())
    }

    async fn on_update_monitor_into_list(self: &Arc<Self>, monitors: MonitorList) -> Result<()> {
        self.monitors.lock().await.extend(monitors);
        Ok(())
    }

    async fn on_event(self: &Arc<Self>, event: Event, payload: Value) -> Result<()> {
        match event {
            Event::MonitorList => {
                self.on_monitor_list(
                    serde_json::from_value(payload)
                        .log_error(module_path!(), |_| "Failed to deserialize MonitorList")
                        .unwrap(),
                )
                .await?
            }
            Event::NotificationList => {
                self.on_notification_list(
                    serde_json::from_value(payload)
                        .log_error(module_path!(), |_| "Failed to deserialize NotificationList")
                        .unwrap(),
                )
                .await?
            }
            Event::MaintenanceList => {
                self.on_maintenance_list(
                    serde_json::from_value(payload)
                        .log_error(module_path!(), |_| "Failed to deserialize MaintenanceList")
                        .unwrap(),
                )
                .await?
            }
            Event::StatusPageList => {
                self.on_status_page_list(
                    serde_json::from_value(payload)
                        .log_error(module_path!(), |_| "Failed to deserialize StatusPageList")
                        .unwrap(),
                )
                .await?
            }
            Event::DockerHostList => {
                self.on_docker_host_list(
                    serde_json::from_value(payload)
                        .log_error(module_path!(), |_| "Failed to deserialize DockerHostList")
                        .unwrap(),
                )
                .await?
            }
            Event::Info => self.on_info().await?,
            Event::AutoLogin => self.on_auto_login().await?,
            Event::LoginRequired => self.on_login_required().await?,
            Event::UpdateMonitorIntoList => {
                self.on_update_monitor_into_list(
                    serde_json::from_value(payload)
                        .log_error(module_path!(), |_| {
                            "Failed to deserialize UpdateMonitorIntoList"
                        })
                        .unwrap(),
                )
                .await?
            }
            Event::DeleteMonitorFromList => {
                self.on_delete_monitor_from_list(payload.as_i64().unwrap().try_into().unwrap())
                    .await?
            }
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
            Ok(LoginResponse::TokenRequired { .. }) => Err(Error::TokenRequired),
            Ok(LoginResponse::Normal {
                ok: true,
                token: Some(auth_token),
                ..
            }) => {
                debug!("Logged in as {}!", username.as_ref());
                *self.is_logged_in.lock().await = true;
                *self.auth_token.lock().await = Some(auth_token);
                Ok(())
            }
            Ok(LoginResponse::Normal {
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
        .log_warn(std::module_path!(), |e| e.to_string())
    }

    pub async fn login_by_token(self: &Arc<Self>, auth_token: impl AsRef<str>) -> Result<()> {
        let result: Result<LoginResponse> = self
            .call("loginByToken", vec![json!(auth_token.as_ref())], "", false)
            .await;

        match result {
            Ok(LoginResponse::TokenRequired { .. }) => Err(Error::TokenRequired),
            Ok(LoginResponse::Normal { ok: true, .. }) => {
                debug!("Logged in using auth_token!");
                *self.is_logged_in.lock().await = true;
                Ok(())
            }
            Ok(LoginResponse::Normal {
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
        .log_warn(std::module_path!(), |e| e.to_string())
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
        let config_json =
            serde_json::to_value(notification.config.clone().unwrap_or_else(|| json!({}))).unwrap();

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

    async fn verify_monitor(self: &Arc<Self>, monitor: &Monitor) -> Result<()> {
        if let Some(referenced_parent) = monitor.common().parent() {
            if !self
                .monitors
                .lock()
                .await
                .values()
                .any(|m| m.common().id().is_some_and(|id| &id == referenced_parent))
            {
                return Err(Error::InvalidReference(
                    InvalidReferenceError::InvalidParent(referenced_parent.to_string()),
                ));
            }
        }

        if let Some(referenced_notifications) = monitor.common().notification_id_list() {
            let available_notifications = self
                .notifications
                .lock()
                .await
                .iter()
                .filter_map(|n| n.id)
                .collect::<HashSet<_>>();

            for (notification_id, _) in referenced_notifications {
                if let Some(id) = notification_id.parse::<i32>().ok() {
                    if !available_notifications.contains(&id) {
                        return Err(Error::InvalidReference(
                            InvalidReferenceError::InvalidNotification(notification_id.to_owned()),
                        ));
                    }
                } else {
                    return Err(Error::InvalidReference(
                        InvalidReferenceError::InvalidNotification(notification_id.to_owned()),
                    ));
                }
            }
        }

        if let Monitor::Docker {
            value: docker_monitor,
        } = monitor
        {
            if let Some(referenced_docker_host) = &docker_monitor.docker_host {
                if !self
                    .docker_hosts
                    .lock()
                    .await
                    .iter()
                    .any(|dh| dh.id.is_some_and(|id| &id == referenced_docker_host))
                {
                    return Err(Error::InvalidReference(
                        InvalidReferenceError::InvalidDockerHost(
                            referenced_docker_host.to_string(),
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    pub async fn add_monitor(self: &Arc<Self>, monitor: &mut Monitor, verify: bool) -> Result<()> {
        if verify {
            self.verify_monitor(monitor).await?;
        }

        let tags = mem::take(monitor.common_mut().tags_mut());
        let notifications = mem::take(monitor.common_mut().notification_id_list_mut());

        #[cfg(feature = "private-api")]
        let parent_name = mem::take(monitor.common_mut().parent_name_mut());
        #[cfg(feature = "private-api")]
        let create_paused = mem::take(monitor.common_mut().create_paused_mut());
        #[cfg(feature = "private-api")]
        let notification_names = mem::take(monitor.common_mut().notification_names_mut());
        #[cfg(feature = "private-api")]
        let docker_host_name = match monitor {
            Monitor::Docker {
                value: docker_monitor,
            } => mem::take(&mut docker_monitor.docker_host_name),
            _ => None,
        };
        #[cfg(feature = "private-api")]
        let tag_names = mem::take(monitor.common_mut().tag_names_mut());

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

        #[cfg(feature = "private-api")]
        {
            *monitor.common_mut().parent_name_mut() = parent_name;
            *monitor.common_mut().create_paused_mut() = create_paused;
            *monitor.common_mut().notification_names_mut() = notification_names;
            if let Monitor::Docker {
                value: docker_monitor,
            } = monitor
            {
                docker_monitor.docker_host_name = docker_host_name;
            }
            *monitor.common_mut().tag_names_mut() = tag_names;
        }

        self.edit_monitor(monitor, false).await?;

        self.monitors
            .lock()
            .await
            .insert(id.to_string(), monitor.clone());

        #[cfg(feature = "private-api")]
        if create_paused == Some(true) {
            self.pause_monitor(id).await?;
        }

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

    pub async fn edit_monitor(self: &Arc<Self>, monitor: &mut Monitor, verify: bool) -> Result<()> {
        if verify {
            self.verify_monitor(monitor).await?;
        }

        let tags = mem::take(monitor.common_mut().tags_mut());

        #[cfg(feature = "private-api")]
        let create_paused = mem::take(monitor.common_mut().create_paused_mut());

        let mut monitor_json = serde_json::to_value(&monitor).unwrap();

        // Workaround for https://github.com/BigBoot/AutoKuma/issues/72 until fixed in UptimeKuma
        if let Some(monitor_json) = monitor_json.as_object_mut() {
            if !monitor_json.contains_key("url") {
                monitor_json.insert("url".to_owned(), json!("https://"));
            }
        }

        let id: i32 = self
            .call("editMonitor", vec![monitor_json], "/monitorID", true)
            .await?;

        self.update_monitor_tags(id, &tags).await?;

        *monitor.common_mut().tags_mut() = tags;

        #[cfg(feature = "private-api")]
        {
            *monitor.common_mut().create_paused_mut() = create_paused;
        }

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
                    .join(&format!("api/status-page/{}", slug))
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
            .log_warn(std::module_path!(), |e| e.to_string())
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
        let mut config = serde_json::to_value(status_page.clone()).unwrap();
        config
            .as_object_mut()
            .unwrap()
            .insert("logo".to_owned(), status_page.icon.clone().into());

        let _: bool = self
            .call(
                "saveStatusPage",
                vec![
                    serde_json::to_value(status_page.slug.clone()).unwrap(),
                    serde_json::to_value(config).unwrap(),
                    serde_json::to_value(status_page.icon.clone()).unwrap(),
                    serde_json::to_value(status_page.public_group_list.clone()).unwrap(),
                ],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn add_docker_host(self: &Arc<Self>, docker_host: &mut DockerHost) -> Result<()> {
        self.edit_docker_host(docker_host).await
    }

    pub async fn edit_docker_host(self: &Arc<Self>, docker_host: &mut DockerHost) -> Result<()> {
        docker_host.id = self
            .call(
                "addDockerHost",
                vec![
                    serde_json::to_value(docker_host.clone()).unwrap(),
                    serde_json::to_value(docker_host.id.clone()).unwrap(),
                ],
                "/id",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn delete_docker_host(self: &Arc<Self>, docker_host_id: i32) -> Result<()> {
        let _: bool = self
            .call(
                "deleteDockerHost",
                vec![serde_json::to_value(docker_host_id).unwrap()],
                "/ok",
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn test_docker_host(self: &Arc<Self>, docker_host: &DockerHost) -> Result<String> {
        let msg: String = self
            .call(
                "testDockerHost",
                vec![serde_json::to_value(docker_host).unwrap()],
                "/msg",
                true,
            )
            .await?;

        Ok(msg)
    }

    pub async fn get_database_size(self: &Arc<Self>) -> Result<u64> {
        let size: u64 = self.call("getDatabaseSize", vec![], "/size", true).await?;
        Ok(size)
    }

    pub async fn shrink_database(self: &Arc<Self>) -> Result<()> {
        let _: bool = self.call("shrinkDatabase", vec![], "/ok", true).await?;
        Ok(())
    }

    pub async fn connect(self: &Arc<Self>) -> Result<()> {
        let mut tls_config = TlsConnector::builder();

        tls_config.danger_accept_invalid_certs(!self.config.tls.verify);

        if let Some((_, cert)) = &self.custom_cert {
            tls_config.add_root_certificate(cert.clone());
        }

        self.is_ready.lock().await.reset();
        *self.is_logged_in.lock().await = false;
        *self.socket_io.lock().await = None;

        let mut builder = ClientBuilder::new(
            self.config
                .url
                .join("socket.io/")
                .map_err(|e| Error::InvalidUrl(e.to_string()))?,
        )
        .tls_config(tls_config.build().map_err(|e| {
            Error::InvalidTlsCert(
                self.custom_cert
                    .as_ref()
                    .map(|(file, _)| file.to_owned())
                    .unwrap_or_default(),
                e.to_string(),
            )
        })?)
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
                                        .log_warn(std::module_path!(), || {
                                            "Error while deserializing Event..."
                                        })
                                        .unwrap_or(""),
                                ) {
                                    handle.clone().spawn(async move {
                                        _ = arc.on_event(e, json!(null)).await.log_warn(
                                            std::module_path!(),
                                            |e| {
                                                format!(
                                                    "Error while handling message event: {}",
                                                    e.to_string()
                                                )
                                            },
                                        );
                                    });
                                }
                            }
                            (event, Payload::Text(params)) => {
                                if let Ok(e) = Event::from_str(&String::from(event)) {
                                    handle.clone().spawn(async move {
                                        _ = arc
                                            .on_event(e.clone(), params.into_iter().next().unwrap())
                                            .await
                                            .log_warn(std::module_path!(), |err| {
                                                format!(
                                                    "Error while handling '{:?}' event: {}",
                                                    e,
                                                    err.to_string()
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
            .log_error(std::module_path!(), |_| "Error during connect")
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
        })
        .await
        .pipe(|result| {
            return match result {
                Ok(_) => Ok(()),
                Err(e) if e.is_cancelled() => Ok(()),
                Err(e) => Err(Error::CommunicationError(format!(
                    "Error while disconnecting: {}",
                    e.to_string()
                ))),
            };
        })
        .log_error(std::module_path!(), |e| e.to_string())?;

        Ok(())
    }

    pub async fn is_ready(self: &Arc<Self>) -> bool {
        self.is_ready.lock().await.is_ready()
    }
}

/// A client for interacting with Uptime Kuma.
///
/// Example:
/// ```
/// // Connect to the server
/// let client = Client::connect(Config {
///         url: Url::parse("http://localhost:3001").expect("Invalid URL"),
///         username: Some("Username".to_owned()),
///         password: Some("Password".to_owned()),
///         ..Default::default()
///     })
///     .await
///     .expect("Failed to connect to server");
///
/// // Create a tag
/// let tag_definition = client
///     .add_tag(TagDefinition {
///         name: Some("example_tag".to_owned()),
///         color: Some("red".to_owned()),
///         ..Default::default()
///     })
///     .await
///     .expect("Failed to add tag");
///
/// // Create a group
/// let group = client
///     .add_monitor(MonitorGroup {
///         name: Some("Example Group".to_owned()),
///         tags: vec![Tag {
///             tag_id: tag_definition.tag_id,
///             value: Some("example_group".to_owned()),
///             ..Default::default()
///         }],
///         ..Default::default()
///     })
///     .await
///     .expect("Failed to add group");
///
/// // Createa a notification
/// let notification = client
///     .add_notification(Notification {
///         name: Some("Example Notification".to_owned()),
///         config: Some(serde_json::json!({
///             "webhookURL": "https://webhook.site/304eeaf2-0248-49be-8985-2c86175520ca",
///             "webhookContentType": "json"
///         })),
///         ..Default::default()
///     })
///     .await
///     .expect("Failed to add notification");
///
/// // Create a monitor
/// client
///     .add_monitor(MonitorHttp {
///         name: Some("Monitor Name".to_owned()),
///         url: Some("https://example.com".to_owned()),
///         parent: group.common().id().clone(),
///         tags: vec![Tag {
///             tag_id: tag_definition.tag_id,
///             value: Some("example_monitor".to_owned()),
///             ..Default::default()
///         }],
///         notification_id_list: Some(
///             vec![(
///                 notification.id.expect("No notification ID").to_string(),
///                 true,
///             )]
///             .into_iter()
///             .collect(),
///         ),
///         ..Default::default()
///     })
///     .await
///     .expect("Failed to add monitor");
///
/// let monitors = client.get_monitors().await.expect("Failed to get monitors");
/// println!("{:?}", monitors);
/// ```
///
pub struct Client {
    worker: Arc<Worker>,
}

impl Client {
    pub async fn connect(config: Config) -> Result<Client> {
        let worker = Worker::new(config)?;
        match worker.connect().await {
            Ok(_) => Ok(Self { worker }),
            Err(e) => {
                _ = worker
                    .disconnect()
                    .await
                    .log_error(std::module_path!(), |e| e.to_string());

                Err(e)
            }
        }
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
    pub async fn add_monitor<T: Into<Monitor>>(&self, monitor: T) -> Result<Monitor> {
        let mut monitor = monitor.into();
        self.worker.add_monitor(&mut monitor, true).await?;
        Ok(monitor)
    }

    /// Edits an existing monitor in Uptime Kuma.
    pub async fn edit_monitor<T: Into<Monitor>>(&self, monitor: T) -> Result<Monitor> {
        let mut monitor = monitor.into();
        self.worker.edit_monitor(&mut monitor, true).await?;
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
    pub async fn get_status_page<T: AsRef<str>>(&self, slug: T) -> Result<StatusPage> {
        self.worker.get_status_page(slug.as_ref()).await
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
    pub async fn delete_status_page<T: AsRef<str>>(&self, slug: T) -> Result<()> {
        self.worker.delete_status_page(slug.as_ref()).await
    }

    /// Retrieves a list of status pages from Uptime Kuma.
    pub async fn get_docker_hosts(&self) -> Result<DockerHostList> {
        match self.worker.is_ready().await {
            true => Ok(self.worker.docker_hosts.lock().await.clone()),
            false => Err(Error::NotReady),
        }
    }

    /// Retrieves information about a specific docker host identified by its id.
    pub async fn get_docker_host(&self, docker_host_id: i32) -> Result<DockerHost> {
        self.get_docker_hosts().await.and_then(|docker_host| {
            docker_host
                .into_iter()
                .find(|docker_host| docker_host.id == Some(docker_host_id))
                .ok_or_else(|| Error::IdNotFound("Docker Host".to_owned(), docker_host_id))
        })
    }

    /// Adds a new docker host to Uptime Kuma.
    pub async fn add_docker_host(&self, mut docker_host: DockerHost) -> Result<DockerHost> {
        self.worker.add_docker_host(&mut docker_host).await?;
        Ok(docker_host)
    }

    /// Edits an existing docker host in Uptime Kuma.
    pub async fn edit_docker_host(&self, mut docker_host: DockerHost) -> Result<DockerHost> {
        self.worker.edit_docker_host(&mut docker_host).await?;
        Ok(docker_host)
    }

    /// Deletes a docker host from Uptime Kuma based on its id.
    pub async fn delete_docker_host(&self, docker_host_id: i32) -> Result<()> {
        self.worker.delete_docker_host(docker_host_id).await
    }

    /// Test a docker host in Uptime Kuma.
    pub async fn test_docker_host<T: std::borrow::Borrow<DockerHost>>(
        &self,
        docker_host: T,
    ) -> Result<String> {
        self.worker.test_docker_host(docker_host.borrow()).await
    }

    /// Get the size of the monitor database (SQLite only)
    pub async fn get_database_size(&self) -> Result<u64> {
        self.worker.get_database_size().await
    }

    /// Trigger database VACUUM for the monitor database (SQLite only)
    pub async fn shrink_database(&self) -> Result<()> {
        self.worker.shrink_database().await
    }

    /// Disconnects the client from Uptime Kuma.
    pub async fn disconnect(&self) -> Result<()> {
        self.worker.disconnect().await
    }

    /// Get the auth token from this client if available.
    pub async fn get_auth_token(&self) -> Option<String> {
        self.worker.auth_token.lock().await.clone()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let worker = self.worker.clone();
        tokio::spawn(async move {
            _ = worker
                .disconnect()
                .await
                .log_error(std::module_path!(), |e| e.to_string());
        });
    }
}
