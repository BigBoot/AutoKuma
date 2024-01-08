use confique::env::parse::list_by_comma;

#[derive(confique::Config)]
pub struct KumaConfig {
    #[config(env = "AUTOKUMA__KUMA__URL")]
    pub url: String,
    #[config(env = "AUTOKUMA__KUMA__USERNAME")]
    pub username: Option<String>,
    #[config(env = "AUTOKUMA__KUMA__PASSWORD")]
    pub password: Option<String>,
    #[config(env = "AUTOKUMA__KUMA__MFA_TOKEN")]
    pub mfa_token: Option<String>,
    #[config(env = "AUTOKUMA__KUMA__HEADER", default = [], parse_env = list_by_comma)]
    pub headers: Vec<String>,
    #[config(env = "AUTOKUMA__KUMA__TAG_NAME", default = "AutoKuma")]
    pub tag_name: String,
    #[config(env = "AUTOKUMA__KUMA__TAG_COLOR", default = "#42C0FB")]
    pub tag_color: String,
    #[config(env = "AUTOKUMA__KUMA__CONNECT_TIMEOUT", default = 30.0)]
    pub connect_timeout: f64,
    #[config(env = "AUTOKUMA__KUMA__CALL_TIMEOUT", default = 30.0)]
    pub call_timeout: f64,
    #[config(env = "AUTOKUMA__KUMA__DEFAULT_SETTINGS", default = "")]
    pub default_settings: String,
}

#[derive(confique::Config)]
pub struct DockerConfig {
    #[config(env = "AUTOKUMA__DOCKER__ENABLED", default = true)]
    pub enabled: bool,
    #[config(
        env = "AUTOKUMA__DOCKER__SOCKET_PATH",
        default = "/var/run/docker.sock"
    )]
    pub socket_path: String,
    #[config(env = "AUTOKUMA__DOCKER__LABEL_PREFIX", default = "kuma")]
    pub label_prefix: String,
}

#[derive(confique::Config)]
pub struct Config {
    #[config(nested)]
    pub kuma: KumaConfig,
    #[config(nested)]
    pub docker: DockerConfig,
    #[config(env = "AUTOKUMA__SYNC_INTERVAL", default = 5.0)]
    pub sync_interval: f64,
    #[config(env = "AUTOKUMA__STATIC_MONITORS", default = "/monitors")]
    pub static_monitors: String,
}
