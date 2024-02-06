use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(serialize_all = "camelCase")]
pub(crate) enum Event {
    ApiKeyList,
    AutoLogin,
    AvgPing,
    CertInfo,
    Connect,
    Disconnect,
    DockerHostList,
    Heartbeat,
    HeartbeatList,
    ImportantHeartbeatList,
    Info,
    InitServerTimezone,
    MaintenanceList,
    MonitorList,
    NotificationList,
    ProxyList,
    StatusPageList,
    Uptime,
}
