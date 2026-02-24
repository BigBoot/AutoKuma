use kuma_client::deserialize::DeserializeVecLenient;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;
use serde_inline_default::serde_inline_default;
use serde_with::{DeserializeAs, PickFirst, SerializeAs, StringWithSeparator, formats::SemicolonSeparator, serde_as};
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
    // Just kept for backwards compatibility, use hosts instead
    #[doc(hidden)]
    #[serde_inline_default(None)]
    pub socket_path: Option<String>,

    /// List of Docker hosts. If set this will override socker_path. Use a semicolon separated string when setting using an env variable.
    #[serde_as(as = "Option<DockerHostsConfig>")]
    #[serde(default)]
    pub hosts: Option<Vec<DockerHostConfig>>,

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

impl<'de> DeserializeAs<'de, Vec<DockerHostConfig>> for DockerHostsConfig {
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<DockerHostConfig>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn host_from_url(url: String) -> DockerHostConfig {
            DockerHostConfig {
                url,
                tls_verify: None,
                tls_cert_path: None,
            }
        }

        fn parse_host_entry<E>(value: serde_json::Value) -> Result<DockerHostConfig, E>
        where
            E: serde::de::Error,
        {
            match value {
                serde_json::Value::String(url) => Ok(host_from_url(url)),
                serde_json::Value::Object(_) => {
                    serde_json::from_value::<DockerHostConfig>(value).map_err(serde::de::Error::custom)
                }
                _ => Err(serde::de::Error::custom("Unable to parse docker host entry")),
            }
        }

        fn parse_hosts_from_value<E>(value: serde_json::Value) -> Result<Vec<DockerHostConfig>, E>
        where
            E: serde::de::Error,
        {
            match value {
                serde_json::Value::String(url) => Ok(vec![host_from_url(url)]),
                serde_json::Value::Object(map) => Ok(vec![parse_host_entry(serde_json::Value::Object(map))?]),
                serde_json::Value::Array(entries) => entries
                    .into_iter()
                    .map(parse_host_entry)
                    .collect::<Result<Vec<_>, _>>(),
                _ => Err(serde::de::Error::custom("Unable to parse docker hosts config")),
            }
        }

        let value = serde_json::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let result = match value {
            serde_json::Value::String(raw) => {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&raw) {
                    parse_hosts_from_value::<D::Error>(json_value)?
                } else {
                    raw.split(';')
                        .filter(|entry| !entry.is_empty())
                        .map(|entry| host_from_url(entry.to_owned()))
                        .collect::<Vec<_>>()
                }
            }
            other => parse_hosts_from_value::<D::Error>(other)?,
        };

        Ok(result)
    }
}

impl SerializeAs<Vec<DockerHostConfig>> for DockerHostsConfig {
    fn serialize_as<S>(source: &Vec<DockerHostConfig>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Vec::<DockerHostConfig>::serialize(source, serializer)
    }
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DockerHostsConfig {
    pub hosts: Vec<DockerHostConfig>,
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DockerHostConfig {
    // The Docker host URL, e.g. tcp://a:2375 or unix:///var/run/docker.sock
    pub url: String,

    // Whether to verify TLS certificates when connecting to this host
    #[serde_inline_default(None)]
    pub tls_verify: Option<bool>,

    // Path to CA certificate for this host, if TLS is enabled
    #[serde_inline_default(None)]
    pub tls_cert_path: Option<String>,
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

#[cfg(test)]
mod tests {
    use ::config::{Config as RawConfig, Environment, File, FileFormat};
    use super::*;
    use serde_json::json;
    use std::{env, sync::Mutex};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn bootstrap_source() -> String {
        serde_json::to_string(&json!({
            "kuma": {"tls": {}},
            "docker": {},
            "files": {},
            "kubernetes": {}
        }))
        .unwrap()
    }

    fn parse_from_formatted_config(config_text: &str, format: FileFormat) -> Config {
        RawConfig::builder()
            .add_source(File::from_str(&bootstrap_source(), FileFormat::Json))
            .add_source(File::from_str(config_text, format))
            .build()
            .unwrap()
            .try_deserialize::<Config>()
            .unwrap()
    }

    fn with_temp_env_vars<F, T>(vars: &[(&str, &str)], run: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _lock = ENV_LOCK.lock().unwrap();
        let previous = vars
            .iter()
            .map(|(key, _)| (key.to_string(), env::var(key).ok()))
            .collect::<Vec<_>>();

        for (key, value) in vars {
            env::set_var(key, value);
        }

        let result = run();

        for (key, value) in previous {
            if let Some(value) = value {
                env::set_var(&key, value);
            } else {
                env::remove_var(&key);
            }
        }

        result
    }

    fn parse_from_environment(vars: &[(&str, &str)]) -> Config {
        with_temp_env_vars(vars, || {
            RawConfig::builder()
                .add_source(File::from_str(&bootstrap_source(), FileFormat::Json))
                .add_source(
                    Environment::with_prefix("AUTOKUMA")
                        .separator("__")
                        .prefix_separator("__"),
                )
                .build()
                .unwrap()
                .try_deserialize::<Config>()
                .unwrap()
        })
    }

    fn base_env_vars() -> Vec<(&'static str, &'static str)> {
        vec![
            ("AUTOKUMA__KUMA__URL", "http://all.local:3001"),
            ("AUTOKUMA__KUMA__USERNAME", "admin"),
            ("AUTOKUMA__KUMA__PASSWORD", "secret"),
            ("AUTOKUMA__KUMA__MFA_TOKEN", "123456"),
            ("AUTOKUMA__KUMA__MFA_SECRET", "MFASECRET"),
            ("AUTOKUMA__KUMA__AUTH_TOKEN", "authtoken"),
            ("AUTOKUMA__KUMA__HEADERS", "X-Test:1,X-Env:2"),
            ("AUTOKUMA__KUMA__CONNECT_TIMEOUT", "10.0"),
            ("AUTOKUMA__KUMA__CALL_TIMEOUT", "20.0"),
            ("AUTOKUMA__KUMA__TLS__VERIFY", "false"),
            ("AUTOKUMA__KUMA__TLS__CERT", "/certs/custom.pem"),
            ("AUTOKUMA__DOCKER__ENABLED", "false"),
            ("AUTOKUMA__DOCKER__SOCKET_PATH", "/var/run/docker.sock"),
            ("AUTOKUMA__DOCKER__SOURCE", "both"),
            ("AUTOKUMA__DOCKER__LABEL_PREFIX", "ak"),
            ("AUTOKUMA__DOCKER__HOSTS", "tcp://host-a:2375;tcp://host-b:2375"),
            ("AUTOKUMA__DOCKER__EXCLUDE_CONTAINER_PATTERNS", "ignore-a;ignore-b"),
            ("AUTOKUMA__FILES__ENABLED", "false"),
            ("AUTOKUMA__FILES__FOLLOW_SYMLINKS", "true"),
            ("AUTOKUMA__KUBERNETES__ENABLED", "true"),
            ("AUTOKUMA__STATIC_MONITORS", "/data/monitors"),
            ("AUTOKUMA__ON_DELETE", "keep"),
            ("AUTOKUMA__DELETE_GRACE_PERIOD", "120.0"),
            ("AUTOKUMA__SYNC_INTERVAL", "42.5"),
            ("AUTOKUMA__TAG_NAME", "AK Tag"),
            ("AUTOKUMA__TAG_COLOR", "#123ABC"),
            ("AUTOKUMA__DATA_PATH", "/data/autokuma"),
            ("AUTOKUMA__DEFAULT_SETTINGS", "max_retries=3"),
            ("AUTOKUMA__SNIPPETS__http", "method=GET"),
            ("AUTOKUMA__SNIPPETS__ping", "packet_size=64"),
            ("AUTOKUMA__LOG_DIR", "/var/log/autokuma"),
            ("AUTOKUMA__INSECURE_ENV_ACCESS", "true"),
        ]
    }

    fn base_env_vars_with(overrides: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
        let mut vars = base_env_vars();

        for (key, value) in overrides {
            if let Some((_, existing)) = vars.iter_mut().find(|(existing_key, _)| existing_key == key) {
                *existing = *value;
            } else {
                vars.push((*key, *value));
            }
        }

        vars
    }

    fn minimal_config_value() -> serde_json::Value {
        json!({
            "kuma": {
                "url": "http://localhost:3001",
                "tls": {}
            },
            "docker": {},
            "kubernetes": {},
            "files": {}
        })
    }

    #[test]
    fn docker_source_deserializes_all_supported_variants() {
        let cases = [
            ("Containers", DockerSource::Containers),
            ("container", DockerSource::Containers),
            ("Services", DockerSource::Services),
            ("service", DockerSource::Services),
            ("Both", DockerSource::Both),
            ("both", DockerSource::Both),
        ];

        for (value, expected) in cases {
            let parsed: DockerSource = serde_json::from_value(json!(value)).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn delete_behavior_deserializes_all_supported_variants() {
        let cases = [
            ("Delete", DeleteBehavior::Delete),
            ("delete", DeleteBehavior::Delete),
            ("Keep", DeleteBehavior::Keep),
            ("keep", DeleteBehavior::Keep),
        ];

        for (value, expected) in cases {
            let parsed: DeleteBehavior = serde_json::from_value(json!(value)).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn docker_hosts_deserializes_from_all_supported_shapes() {
        let from_semicolon: DockerConfig = serde_json::from_value(json!({
            "hosts": "tcp://a:2375;unix:///var/run/docker.sock"
        }))
        .unwrap();

        assert_eq!(
            from_semicolon.hosts,
            Some(vec![
                DockerHostConfig {
                    url: "tcp://a:2375".to_owned(),
                    tls_verify: None,
                    tls_cert_path: None,
                },
                DockerHostConfig {
                    url: "unix:///var/run/docker.sock".to_owned(),
                    tls_verify: None,
                    tls_cert_path: None,
                }
            ])
        );

        let from_array: DockerConfig = serde_json::from_value(json!({
            "hosts": [
                {
                    "url": "tcp://a:2375",
                    "tls_verify": true,
                    "tls_cert_path": "/certs/a.pem"
                },
                {
                    "url": "unix:///var/run/docker.sock"
                }
            ]
        }))
        .unwrap();

        assert_eq!(
            from_array.hosts,
            Some(vec![
                DockerHostConfig {
                    url: "tcp://a:2375".to_owned(),
                    tls_verify: Some(true),
                    tls_cert_path: Some("/certs/a.pem".to_owned()),
                },
                DockerHostConfig {
                    url: "unix:///var/run/docker.sock".to_owned(),
                    tls_verify: None,
                    tls_cert_path: None,
                },
            ])
        );

        let from_array_of_strings: DockerConfig = serde_json::from_value(json!({
            "hosts": ["tcp://a:2375", "unix:///var/run/docker.sock"]
        }))
        .unwrap();

        assert_eq!(from_array_of_strings.hosts, from_semicolon.hosts);

        let from_wrapped_object = serde_json::from_value::<DockerConfig>(json!({
            "hosts": {
                "hosts": ["tcp://a:2375", "unix:///var/run/docker.sock"]
            }
        }));

        assert!(from_wrapped_object.is_err());
    }

    #[test]
    fn docker_hosts_rejects_invalid_values() {
        let result = serde_json::from_value::<DockerConfig>(json!({ "hosts": 123 }));
        assert!(result.is_err());
    }

    #[test]
    fn exclude_container_patterns_deserializes_from_all_supported_shapes() {
        let from_array: DockerConfig = serde_json::from_value(json!({
            "exclude_container_patterns": ["a", "b"]
        }))
        .unwrap();
        assert_eq!(from_array.exclude_container_patterns, Some(vec!["a".to_owned(), "b".to_owned()]));

        let from_json_array_string: DockerConfig = serde_json::from_value(json!({
            "exclude_container_patterns": "[\"a\",\"b\"]"
        }))
        .unwrap();
        assert_eq!(
            from_json_array_string.exclude_container_patterns,
            Some(vec!["a".to_owned(), "b".to_owned()])
        );

        let from_semicolon_string: DockerConfig = serde_json::from_value(json!({
            "exclude_container_patterns": "a;b"
        }))
        .unwrap();
        assert_eq!(
            from_semicolon_string.exclude_container_patterns,
            Some(vec!["a".to_owned(), "b".to_owned()])
        );
    }

    #[test]
    fn config_deserializes_screaming_snake_case_aliases() {
        let parsed: Config = serde_json::from_value(json!({
            "KUMA": {
                "URL": "http://localhost:3001",
                "TLS": {
                    "VERIFY": false
                }
            },
            "DOCKER": {
                "SOURCE": "service",
                "LABEL_PREFIX": "ak",
                "EXCLUDE_CONTAINER_PATTERNS": "x;y"
            },
            "KUBERNETES": {
                "ENABLED": true
            },
            "FILES": {
                "FOLLOW_SYMLINKS": true
            },
            "SYNC_INTERVAL": 7.5,
            "ON_DELETE": "keep",
            "INSECURE_ENV_ACCESS": true
        }))
        .unwrap();

        assert_eq!(parsed.docker.source, DockerSource::Services);
        assert_eq!(parsed.docker.label_prefix, "ak");
        assert_eq!(parsed.docker.exclude_container_patterns, Some(vec!["x".to_owned(), "y".to_owned()]));
        assert!(parsed.kubernetes.enabled);
        assert!(parsed.files.follow_symlinks);
        assert_eq!(parsed.sync_interval, 7.5);
        assert_eq!(parsed.on_delete, DeleteBehavior::Keep);
        assert!(parsed.insecure_env_access);
        assert!(!parsed.kuma.tls.verify);
    }

    #[test]
    fn config_round_trip_serialization_preserves_values() {
        let original: Config = serde_json::from_value(json!({
            "kuma": {
                "url": "http://localhost:3001",
                "tls": {}
            },
            "docker": {
                "source": "both",
                "hosts": "tcp://a:2375;tcp://b:2375",
                "exclude_container_patterns": "a;b"
            },
            "kubernetes": {
                "enabled": true
            },
            "files": {
                "enabled": true,
                "follow_symlinks": true
            },
            "sync_interval": 9.0,
            "on_delete": "keep",
            "tag_name": "AutoKuma",
            "tag_color": "#42C0FB",
            "snippets": {
                "foo": "bar"
            }
        }))
        .unwrap();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, original);
    }

    #[test]
    fn config_deserializes_defaults_for_optional_fields() {
        let parsed: Config = serde_json::from_value(minimal_config_value()).unwrap();

        assert_eq!(parsed.docker.source, DockerSource::Containers);
        assert_eq!(parsed.on_delete, DeleteBehavior::Delete);
        assert_eq!(parsed.sync_interval, 5.0);
        assert_eq!(parsed.delete_grace_period, 60.0);
        assert_eq!(parsed.tag_name, "AutoKuma");
        assert_eq!(parsed.tag_color, "#42C0FB");
        assert_eq!(parsed.default_settings, "");
        assert_eq!(parsed.snippets, HashMap::new());
        assert!(!parsed.insecure_env_access);
    }

    fn assert_all_settings(parsed: &Config, include_host_tls_fields: bool) {
        assert_eq!(parsed.kuma.url.as_str(), "http://all.local:3001/");
        assert_eq!(parsed.kuma.username.as_deref(), Some("admin"));
        assert_eq!(parsed.kuma.password.as_deref(), Some("secret"));
        assert_eq!(parsed.kuma.mfa_token.as_deref(), Some("123456"));
        assert_eq!(parsed.kuma.mfa_secret.as_deref(), Some("MFASECRET"));
        assert_eq!(parsed.kuma.auth_token.as_deref(), Some("authtoken"));
        assert_eq!(parsed.kuma.headers, vec!["X-Test:1".to_owned(), "X-Env:2".to_owned()]);
        assert_eq!(parsed.kuma.connect_timeout, 10.0);
        assert_eq!(parsed.kuma.call_timeout, 20.0);
        assert!(!parsed.kuma.tls.verify);
        assert_eq!(parsed.kuma.tls.cert.as_deref(), Some("/certs/custom.pem"));

        assert!(!parsed.docker.enabled);
        assert_eq!(parsed.docker.socket_path.as_deref(), Some("/var/run/docker.sock"));
        assert_eq!(
            parsed.docker.hosts,
            Some(if include_host_tls_fields {
                vec![
                    DockerHostConfig {
                        url: "tcp://host-a:2375".to_owned(),
                        tls_verify: Some(true),
                        tls_cert_path: Some("/certs/host-a.pem".to_owned()),
                    },
                    DockerHostConfig {
                        url: "tcp://host-b:2375".to_owned(),
                        tls_verify: Some(false),
                        tls_cert_path: Some("/certs/host-b.pem".to_owned()),
                    },
                ]
            } else {
                vec![
                    DockerHostConfig {
                        url: "tcp://host-a:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                    DockerHostConfig {
                        url: "tcp://host-b:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                ]
            })
        );
        assert_eq!(parsed.docker.source, DockerSource::Both);
        assert_eq!(parsed.docker.label_prefix, "ak");
        assert_eq!(
            parsed.docker.exclude_container_patterns,
            Some(vec!["ignore-a".to_owned(), "ignore-b".to_owned()])
        );

        assert!(parsed.kubernetes.enabled);
        assert!(!parsed.files.enabled);
        assert!(parsed.files.follow_symlinks);

        assert_eq!(parsed.sync_interval, 42.5);
        assert_eq!(parsed.static_monitors.as_deref(), Some("/data/monitors"));
        assert_eq!(parsed.on_delete, DeleteBehavior::Keep);
        assert_eq!(parsed.delete_grace_period, 120.0);
        assert_eq!(parsed.tag_name, "AK Tag");
        assert_eq!(parsed.tag_color, "#123ABC");
        assert_eq!(parsed.data_path.as_deref(), Some("/data/autokuma"));
        assert_eq!(parsed.default_settings, "max_retries=3");
        assert_eq!(
            parsed.snippets,
            HashMap::from([
                ("http".to_owned(), "method=GET".to_owned()),
                ("ping".to_owned(), "packet_size=64".to_owned()),
            ])
        );
        assert_eq!(parsed.log_dir.as_deref(), Some("/var/log/autokuma"));
        assert!(parsed.insecure_env_access);
    }

    #[test]
    fn config_parses_all_settings_from_json() {
        let parsed = parse_from_formatted_config(
            r##"{
                "kuma": {
                    "url": "http://all.local:3001",
                    "username": "admin",
                    "password": "secret",
                    "mfa_token": "123456",
                    "mfa_secret": "MFASECRET",
                    "auth_token": "authtoken",
                    "headers": ["X-Test:1", "X-Env:2"],
                    "connect_timeout": 10.0,
                    "call_timeout": 20.0,
                    "tls": {
                        "verify": false,
                        "cert": "/certs/custom.pem"
                    }
                },
                "docker": {
                    "enabled": false,
                    "socket_path": "/var/run/docker.sock",
                    "source": "both",
                    "label_prefix": "ak",
                    "hosts": [
                        {
                            "url": "tcp://host-a:2375",
                            "tls_verify": true,
                            "tls_cert_path": "/certs/host-a.pem"
                        },
                        {
                            "url": "tcp://host-b:2375",
                            "tls_verify": false,
                            "tls_cert_path": "/certs/host-b.pem"
                        }
                    ],
                    "exclude_container_patterns": "ignore-a;ignore-b"
                },
                "files": {
                    "enabled": false,
                    "follow_symlinks": true
                },
                "kubernetes": { "enabled": true },
                "static_monitors": "/data/monitors",
                "on_delete": "keep",
                "delete_grace_period": 120.0,
                "sync_interval": 42.5,
                "tag_name": "AK Tag",
                "tag_color": "#123ABC",
                "data_path": "/data/autokuma",
                "default_settings": "max_retries=3",
                "snippets": {
                    "http": "method=GET",
                    "ping": "packet_size=64"
                },
                "log_dir": "/var/log/autokuma",
                "insecure_env_access": true
            }"##,
            FileFormat::Json,
        );

        assert_all_settings(&parsed, true);
    }

    #[test]
    fn config_parses_all_settings_from_toml() {
        let parsed = parse_from_formatted_config(
            r##"
            on_delete = "keep"
            sync_interval = 42.5
            static_monitors = "/data/monitors"
            delete_grace_period = 120.0
            tag_name = "AK Tag"
            tag_color = "#123ABC"
            data_path = "/data/autokuma"
            default_settings = "max_retries=3"
            log_dir = "/var/log/autokuma"
            insecure_env_access = true

            [kuma]
            url = "http://all.local:3001"
            username = "admin"
            password = "secret"
            mfa_token = "123456"
            mfa_secret = "MFASECRET"
            auth_token = "authtoken"
            headers = ["X-Test:1", "X-Env:2"]
            connect_timeout = 10.0
            call_timeout = 20.0

            [kuma.tls]
            verify = false
            cert = "/certs/custom.pem"

            [docker]
            enabled = false
            socket_path = "/var/run/docker.sock"
            source = "both"
            label_prefix = "ak"
            exclude_container_patterns = "ignore-a;ignore-b"

            [[docker.hosts]]
            url = "tcp://host-a:2375"
            tls_verify = true
            tls_cert_path = "/certs/host-a.pem"

            [[docker.hosts]]
            url = "tcp://host-b:2375"
            tls_verify = false
            tls_cert_path = "/certs/host-b.pem"

            [files]
            enabled = false
            follow_symlinks = true

            [kubernetes]
            enabled = true

            [snippets]
            http = "method=GET"
            ping = "packet_size=64"
            "##,
            FileFormat::Toml,
        );

        assert_all_settings(&parsed, true);
    }

    #[test]
    fn config_parses_legacy_toml_docker_hosts_string_array() {
        let parsed = parse_from_formatted_config(
            r##"
            [kuma]
            url = "http://localhost:3001"

            [kuma.tls]
            verify = true

            [docker]
            enabled = true
            hosts = ["unix://./docker.sock"]
            source = "container"

            [files]
            enabled = true

            [kubernetes]
            enabled = false
            "##,
            FileFormat::Toml,
        );

        assert_eq!(parsed.docker.enabled, true);
        assert_eq!(
            parsed.docker.hosts,
            Some(vec![DockerHostConfig {
                url: "unix://./docker.sock".to_owned(),
                tls_verify: None,
                tls_cert_path: None,
            }])
        );
        assert_eq!(parsed.docker.source, DockerSource::Containers);
    }

    #[test]
    fn config_parses_all_settings_from_yaml() {
        let parsed = parse_from_formatted_config(
            r##"
                        {
                            "kuma": {
                                "url": "http://all.local:3001",
                                "username": "admin",
                                "password": "secret",
                                "mfa_token": "123456",
                                "mfa_secret": "MFASECRET",
                                "auth_token": "authtoken",
                                "headers": ["X-Test:1", "X-Env:2"],
                                "connect_timeout": 10.0,
                                "call_timeout": 20.0,
                                "tls": {
                                    "verify": false,
                                    "cert": "/certs/custom.pem"
                                }
                            },
                            "docker": {
                                "enabled": false,
                                "socket_path": "/var/run/docker.sock",
                                "source": "both",
                                "label_prefix": "ak",
                                "hosts": [
                                    {
                                        "url": "tcp://host-a:2375",
                                        "tls_verify": true,
                                        "tls_cert_path": "/certs/host-a.pem"
                                    },
                                    {
                                        "url": "tcp://host-b:2375",
                                        "tls_verify": false,
                                        "tls_cert_path": "/certs/host-b.pem"
                                    }
                                ],
                                "exclude_container_patterns": "ignore-a;ignore-b"
                            },
                            "files": {
                                "enabled": false,
                                "follow_symlinks": true
                            },
                            "kubernetes": { "enabled": true },
                            "sync_interval": 42.5,
                            "static_monitors": "/data/monitors",
                            "on_delete": "keep",
                            "delete_grace_period": 120.0,
                            "tag_name": "AK Tag",
                            "tag_color": "#123ABC",
                            "data_path": "/data/autokuma",
                            "default_settings": "max_retries=3",
                            "snippets": {
                                "http": "method=GET",
                                "ping": "packet_size=64"
                            },
                            "log_dir": "/var/log/autokuma",
                            "insecure_env_access": true
                        }
            "##,
            FileFormat::Yaml,
        );

        assert_all_settings(&parsed, true);
    }

    #[test]
    fn config_parses_all_settings_from_environment_variables() {
        let vars = base_env_vars();
        let parsed = parse_from_environment(&vars);

        assert_all_settings(&parsed, false);
    }

    #[test]
    fn config_parses_environment_variable_variants() {
        let cases = [
            (
                &[ 
                    ("AUTOKUMA__KUMA__HEADERS", "[\"X-Test:1\",\"X-Env:2\"]"),
                    (
                        "AUTOKUMA__DOCKER__EXCLUDE_CONTAINER_PATTERNS",
                        "[\"ignore-a\",\"ignore-b\"]",
                    ),
                    ("AUTOKUMA__DOCKER__SOURCE", "Both"),
                    ("AUTOKUMA__ON_DELETE", "Keep"),
                ][..],
                DockerSource::Both,
                DeleteBehavior::Keep,
                Some(vec![
                    DockerHostConfig {
                        url: "tcp://host-a:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                    DockerHostConfig {
                        url: "tcp://host-b:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                ]),
            ),
            (
                &[
                    ("AUTOKUMA__KUMA__HEADERS", "X-Test:1,X-Env:2"),
                    ("AUTOKUMA__DOCKER__EXCLUDE_CONTAINER_PATTERNS", "ignore-a;ignore-b"),
                    ("AUTOKUMA__DOCKER__SOURCE", "container"),
                    ("AUTOKUMA__ON_DELETE", "delete"),
                ][..],
                DockerSource::Containers,
                DeleteBehavior::Delete,
                Some(vec![
                    DockerHostConfig {
                        url: "tcp://host-a:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                    DockerHostConfig {
                        url: "tcp://host-b:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                ]),
            ),
            (
                &[
                    (
                        "AUTOKUMA__DOCKER__HOSTS",
                        "[{\"url\":\"tcp://host-a:2375\",\"tls_verify\":true,\"tls_cert_path\":\"/certs/host-a.pem\"},{\"url\":\"tcp://host-b:2375\",\"tls_verify\":false,\"tls_cert_path\":\"/certs/host-b.pem\"}]",
                    ),
                ][..],
                DockerSource::Both,
                DeleteBehavior::Keep,
                Some(vec![
                    DockerHostConfig {
                        url: "tcp://host-a:2375".to_owned(),
                        tls_verify: Some(true),
                        tls_cert_path: Some("/certs/host-a.pem".to_owned()),
                    },
                    DockerHostConfig {
                        url: "tcp://host-b:2375".to_owned(),
                        tls_verify: Some(false),
                        tls_cert_path: Some("/certs/host-b.pem".to_owned()),
                    },
                ]),
            ),
            (
                &[
                    (
                        "AUTOKUMA__DOCKER__HOSTS",
                        "[\"tcp://host-a:2375\",\"tcp://host-b:2375\"]",
                    ),
                ][..],
                DockerSource::Both,
                DeleteBehavior::Keep,
                Some(vec![
                    DockerHostConfig {
                        url: "tcp://host-a:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                    DockerHostConfig {
                        url: "tcp://host-b:2375".to_owned(),
                        tls_verify: None,
                        tls_cert_path: None,
                    },
                ]),
            ),
        ];

        for (overrides, expected_source, expected_on_delete, expected_hosts) in cases {
            let vars = base_env_vars_with(overrides);
            let parsed = parse_from_environment(&vars);

            assert_eq!(parsed.kuma.headers, vec!["X-Test:1".to_owned(), "X-Env:2".to_owned()]);
            assert_eq!(
                parsed.docker.exclude_container_patterns,
                Some(vec!["ignore-a".to_owned(), "ignore-b".to_owned()])
            );
            assert_eq!(parsed.docker.source, expected_source);
            assert_eq!(parsed.on_delete, expected_on_delete);
            assert_eq!(parsed.docker.hosts, expected_hosts);
            assert_eq!(parsed.kuma.url.as_str(), "http://all.local:3001/");
        }
    }

}