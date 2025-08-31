use kuma_client::deserialize::DeserializeVecLenient;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;
use serde_inline_default::serde_inline_default;
use serde_with::{formats::SemicolonSeparator, serde_as, PickFirst, StringWithSeparator};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DockerSource {
    #[serde(alias = "container")]
    Containers,
    #[serde(alias = "service")]
    Services,
    #[serde(alias = "both")]
    Both,
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DockerConfig {
    /// Whether docker integration should be enabled or not.
    #[serde_inline_default(true)]
    pub enabled: bool,

    /// Path to the Docker socket. If not set, the DOCKER_HOST will be used.
    #[serde_inline_default(None)]
    pub socket_path: Option<String>,

    /// List of Docker hosts. If set this will override socker_path. Use a semicolon separated string when setting using an env variable.
    #[serde_as(
        as = "Option<PickFirst<(DeserializeVecLenient<String>, StringWithSeparator::<SemicolonSeparator, String>)>>"
    )]
    #[serde(default)]
    pub hosts: Option<Vec<String>>,

    /// Whether monitors should be created from container or service labels (or both).
    #[serde_inline_default(DockerSource::Containers)]
    pub source: DockerSource,

    /// Prefix used when scanning for container labels.
    #[serde_inline_default("kuma".to_owned())]
    pub label_prefix: String,

    /// Regex patterns to exclude containers by name (semicolon-separated).
    #[serde_as(
        as = "Option<PickFirst<(DeserializeVecLenient<String>, StringWithSeparator::<SemicolonSeparator, String>)>>"
    )]
    #[serde(default)]
    pub exclude_container_patterns: Option<Vec<String>>,
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Whether kubernetes integration should be enabled or not.
    #[serde_inline_default(false)]
    pub enabled: bool,
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FilesConfig {
    /// Whether files source should be enabled or not.
    #[serde_inline_default(true)]
    pub enabled: bool,

    /// Whether the files source should follow symlinks or not.
    #[serde_inline_default(false)]
    pub follow_symlinks: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeleteBehavior {
    #[serde(alias = "delete")]
    Delete,
    #[serde(alias = "keep")]
    Keep,
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub kuma: kuma_client::Config,

    pub docker: DockerConfig,

    pub kubernetes: KubernetesConfig,

    pub files: FilesConfig,

    /// The interval in between syncs.
    #[serde_inline_default(5.0)]
    pub sync_interval: f64,

    /// The path to the folder in which AutoKuma will search for static Monitor definitions.
    #[serde_inline_default(None)]
    pub static_monitors: Option<String>,

    /// Specify what to do when a monitor with given autokuma id is not found anymore.
    #[serde_inline_default(DeleteBehavior::Delete)]
    pub on_delete: DeleteBehavior,

    /// The grace period in seconds before a missing entity is deleted.
    #[serde_inline_default(60.0)]
    pub delete_grace_period: f64,

    /// The name of the AutoKuma tag, used to track managed containers
    #[serde_inline_default("AutoKuma".to_owned())]
    pub tag_name: String,

    /// The color of the AutoKuma tag
    #[serde_inline_default("#42C0FB".to_owned())]
    pub tag_color: String,

    /// Where to store application data
    #[serde_inline_default(None)]
    pub data_path: Option<String>,

    /// Default settings applied to all generated Monitors.
    #[serde_inline_default("".to_owned())]
    pub default_settings: String,

    /// Default settings applied to all generated Monitors.
    #[serde_inline_default(HashMap::new())]
    pub snippets: HashMap<String, String>,

    /// A directory where log files should be stored
    #[serde_inline_default(None)]
    pub log_dir: Option<String>,

    /// Allow access to all env variables in templates, by default only variables starting with AUTOKUMA__ENV__ can be accessed.
    #[serde_inline_default(false)]
    pub insecure_env_access: bool,
}
