use crate::deserialize::DeserializeVecLenient;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;
use serde_inline_default::serde_inline_default;
use serde_with::{formats::CommaSeparator, serde_as, PickFirst, StringWithSeparator};
use url::Url;

/// TLS Configuration for the [Client](crate::Client).
#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[serde_as]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Whether to verify the TLS certificate or not.
    ///
    /// Defaults to `true`.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If
    /// invalid certificates are trusted, *any* certificate for *any* site
    /// will be trusted for use. This includes expired certificates. This
    /// introduces significant vulnerabilities, and should only be used
    /// as a last resort.
    #[serde_inline_default(true)]
    pub verify: bool,

    /// The path to a custom tls certificate in PEM format.
    ///
    /// This can be used to connect to a server that has a self-signed
    /// certificate for example.
    #[serde(default)]
    pub cert: Option<String>,
}

/// Configuration for the [Client](crate::Client).
#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The URL for connecting to Uptime Kuma.
    pub url: Url,

    /// The username for logging into Uptime Kuma (required unless auth is disabled).                      .
    pub username: Option<String>,

    /// The password for logging into Uptime Kuma (required unless auth is disabled).
    pub password: Option<String>,

    /// The MFA token for logging into Uptime Kuma (required if MFA is enabled).
    pub mfa_token: Option<String>,

    /// The MFA secret. Used to generate a tokens for logging into Uptime Kuma (alternative to a single_use mfa_token).
    pub mfa_secret: Option<String>,

    /// JWT Auth token received after a succesfull login, can be used to as an alternative to username/password.
    pub auth_token: Option<String>,

    /// List of HTTP headers to send when connecting to Uptime Kuma.
    #[serde_as(
        as = "PickFirst<(DeserializeVecLenient<String>, StringWithSeparator::<CommaSeparator, String>)>"
    )]
    #[serde(default)]
    pub headers: Vec<String>,

    /// The timeout for the initial connection to Uptime Kuma.
    #[serde_inline_default(30.0)]
    pub connect_timeout: f64,

    /// The timeout for executing calls to the Uptime Kuma server.
    #[serde_inline_default(30.0)]
    pub call_timeout: f64,

    /// How long to wait, after the socket is open, for Uptime Kuma to finish
    /// sending its initial state (monitor list, notifications, ...) before the
    /// connection attempt is given up. A busy server, e.g. one that just
    /// restarted while many clients reconnect, can need well over the
    /// previous fixed ~9 seconds.
    #[serde_inline_default(9.0)]
    pub ready_timeout: f64,

    /// TLS Configuration for the [Client](crate::Client).
    pub tls: TlsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: Url::parse("http://localhost:3001").unwrap(),
            username: None,
            password: None,
            mfa_token: None,
            mfa_secret: None,
            auth_token: None,
            headers: Vec::new(),
            connect_timeout: 30.0,
            call_timeout: 30.0,
            ready_timeout: 9.0,
            tls: TlsConfig::default(),
        }
    }
}

pub mod source {

    use config::{ConfigError, Map, Source, Value, ValueKind};
    use std::env;
    use std::ffi::OsString;

    type Result<T> = std::result::Result<T, ConfigError>;

    /// An environment source collects a dictionary of environment variables values into a hierarchical
    /// config Value type. We have to be aware how the config tree is created from the environment
    /// dictionary, therefore we are mindful about prefixes for the environment keys, level separators,
    /// encoding form (kebab, snake case) etc.
    ///
    /// For prefixes take a look at [`with_prefix`](Environment::with_prefix()).
    /// For level separators take a look at [`separator`](Environment::separator()).
    #[must_use]
    #[derive(Clone, Debug, Default)]
    pub struct Environment {
        /// Optional prefix that will limit access to the environment to only keys that
        /// begin with the defined prefix.
        ///
        /// The prefix is tested to be present on each key before it's considered to be part of the
        /// source environment. The separator character can be set through
        /// [`prefix_separator`](Environment::prefix_separator()).
        ///
        /// For example, the key `CONFIG_DEBUG` would become `DEBUG` with a prefix of `config`.
        prefix: Option<String>,

        /// Optional character sequence that separates the prefix from the rest of the key.
        ///
        /// Defaults to [`separator`](Environment::separator()) if that is set, otherwise `_`.
        prefix_separator: Option<String>,

        /// Optional character sequence that separates each key segment in an environment key pattern.
        /// Consider a nested configuration such as `redis.password`, a separator of `_` would allow
        /// an environment key of `REDIS_PASSWORD` to match.
        ///
        /// If unset, `.` (a dot) is used. In such case `REDIS.PASSWORD` would be the correct key
        /// for the example above.
        separator: Option<String>,

        /// Ignore empty env values (treat as unset).
        ignore_empty: bool,

        // Preserve the prefix while parsing
        keep_prefix: bool,

        /// When enabled, values of the form `@/path/to/file` are replaced by the contents of that
        /// file (trailing newline stripped). Use `@@` to escape a literal leading `@`.
        secret_files: bool,

        source: Option<Map<String, String>>,
    }

    impl Environment {
        /// Optional prefix that will limit access to the environment to only keys that
        /// begin with the defined prefix.
        ///
        /// The prefix is tested to be present on each key before it's considered to be part of the
        /// source environment. The separator character can be set through
        /// [`prefix_separator`](Environment::prefix_separator()).
        ///
        /// For example, the key `CONFIG_DEBUG` would become `DEBUG` with a prefix of `config`.
        pub fn with_prefix(s: &str) -> Self {
            Self {
                prefix: Some(s.into()),
                ..Self::default()
            }
        }

        /// See [`Environment::with_prefix`]
        pub fn prefix(mut self, s: &str) -> Self {
            self.prefix = Some(s.into());
            self
        }

        /// Optional character sequence that separates the prefix from the rest of the key.
        ///
        /// Defaults to [`separator`](Environment::separator()) if that is set, otherwise `_`.
        pub fn prefix_separator(mut self, s: &str) -> Self {
            self.prefix_separator = Some(s.into());
            self
        }

        /// Optional character sequence that separates each key segment in an environment key pattern.
        /// Consider a nested configuration such as `redis.password`, a separator of `_` would allow
        /// an environment key of `REDIS_PASSWORD` to match.
        ///
        /// If unset, `.` (a dot) is used. In such case `REDIS.PASSWORD` would be the correct key
        /// for the example above.
        pub fn separator(mut self, s: &str) -> Self {
            self.separator = Some(s.into());
            self
        }

        /// Ignore empty env values (treat as unset).
        pub fn ignore_empty(mut self, ignore: bool) -> Self {
            self.ignore_empty = ignore;
            self
        }

        // Preserve the prefix while parsing
        pub fn keep_prefix(mut self, keep: bool) -> Self {
            self.keep_prefix = keep;
            self
        }

        pub fn source(mut self, source: Option<Map<String, String>>) -> Self {
            self.source = source;
            self
        }

        /// When enabled, any value starting with `@/` is treated as an absolute file path and
        /// replaced by the file's contents (trailing newline stripped). Use `@@` to produce a
        /// literal leading `@`.
        pub fn secret_files(mut self, enable: bool) -> Self {
            self.secret_files = enable;
            self
        }
    }

    fn resolve_file_ref(value: String, key: &str) -> Result<String> {
        if let Some(rest) = value.strip_prefix("@@/") {
            return Ok(format!("@/{rest}"));
        }
        if let Some(path) = value.strip_prefix("@/") {
            let path = format!("/{path}");
            return std::fs::read_to_string(&path)
                .map(|s| {
                    // Strip exactly one trailing newline (handles \r\n and \n).
                    s.strip_suffix("\r\n")
                        .or_else(|| s.strip_suffix('\n'))
                        .unwrap_or(&s)
                        .to_owned()
                })
                .map_err(|e| {
                    ConfigError::Message(format!(
                        "failed to read secret file {path:?} for config key {key:?}: {e}"
                    ))
                });
        }
        Ok(value)
    }

    fn normalize_key_segments(key: &str, separator: &str) -> Vec<String> {
        if separator.is_empty() {
            return vec![key.to_lowercase()];
        }

        key.split(separator)
            .filter(|segment| !segment.is_empty())
            .map(|s| s.to_lowercase())
            .collect()
    }

    fn insert_value(
        map: &mut Map<String, Value>,
        key: String,
        value: Value,
        separator: &str,
        uri: &String,
    ) -> Result<()> {
        if separator.is_empty() {
            map.insert(key.to_lowercase(), value);
            return Ok(());
        }

        let segments = normalize_key_segments(&key, separator);

        if segments.is_empty() {
            map.insert(key.to_lowercase(), value);
            return Ok(());
        }

        if segments.len() <= 1 {
            if let Some(existing) = map.get(&segments[0]) {
                if matches!(existing.kind, ValueKind::Table(_)) {
                    return Err(ConfigError::Message(format!(
                        "Conflicting config definition for {key:?}."
                    )));
                }
            }

            map.insert(segments[0].clone(), value);
            return Ok(());
        }

        insert_nested_segments(map, &segments, value, &key, uri)
    }

    fn insert_nested_segments(
        map: &mut Map<String, Value>,
        segments: &[String],
        value: Value,
        full_key: &str,
        uri: &String,
    ) -> Result<()> {
        let current = segments[0].clone();

        if segments.len() == 1 {
            if let Some(existing) = map.get(&current) {
                if matches!(existing.kind, ValueKind::Table(_)) {
                    return Err(ConfigError::Message(format!(
                        "Conflicting config definition for {full_key:?}."
                    )));
                }
            }

            map.insert(current, value);
            return Ok(());
        }

        if let Some(existing) = map.get_mut(&current) {
            match &mut existing.kind {
                ValueKind::Table(table) => {
                    return insert_nested_segments(table, &segments[1..], value, full_key, uri);
                }
                _ => {
                    return Err(ConfigError::Message(format!(
                        "Conflicting config definition for {full_key:?}."
                    )));
                }
            }
        }

        map.insert(
            current.clone(),
            Value::new(Some(uri), ValueKind::Table(Map::new())),
        );

        let Some(entry) = map.get_mut(&current) else {
            unreachable!();
        };

        let ValueKind::Table(table) = &mut entry.kind else {
            unreachable!();
        };

        insert_nested_segments(table, &segments[1..], value, full_key, uri)
    }

    impl Source for Environment {
        fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
            Box::new((*self).clone())
        }

        fn collect(&self) -> Result<Map<String, Value>> {
            let mut m = Map::new();
            let uri: String = "the environment".into();

            let separator = self.separator.as_deref().unwrap_or("");

            let prefix_separator =
                match (self.prefix_separator.as_deref(), self.separator.as_deref()) {
                    (Some(pre), _) => pre,
                    (None, Some(sep)) => sep,
                    (None, None) => "_",
                };

            // Define a prefix pattern to test and exclude from keys
            let prefix_pattern = self
                .prefix
                .as_ref()
                .map(|prefix| format!("{prefix}{prefix_separator}"));

            let collector = |(key, value): (OsString, OsString)| {
                let mut key = match key.into_string() {
                    Ok(key) => key,
                    // Key is not valid unicode, skip it
                    Err(_) => return Ok(()),
                };

                // Treat empty environment variables as unset
                if self.ignore_empty && value.is_empty() {
                    return Ok(());
                }

                let mut normalized_key = key.to_lowercase();

                // Check for prefix
                if let Some(ref prefix_pattern) = prefix_pattern {
                    let normalized_prefix_pattern = prefix_pattern.to_lowercase();

                    if normalized_key.starts_with(&normalized_prefix_pattern) {
                        if !self.keep_prefix {
                            // Remove this prefix from the key
                            key = key[prefix_pattern.len()..].to_string();
                            normalized_key =
                                normalized_key[normalized_prefix_pattern.len()..].to_string();
                        }
                    } else {
                        // Skip this key
                        return Ok(());
                    }
                }

                // At this point, we don't know if the key is required or not.
                // Therefore if the value is not a valid unicode string, we error out.
                let value = value.into_string().map_err(|os_string| {
                    ConfigError::Message(format!(
                        "env variable {normalized_key:?} contains non-Unicode data: {os_string:?}"
                    ))
                })?;

                let value = if self.secret_files {
                    resolve_file_ref(value, &normalized_key)?
                } else {
                    value
                };

                insert_value(
                    &mut m,
                    key,
                    Value::new(Some(&uri), ValueKind::String(value)),
                    separator,
                    &uri,
                )?;

                Ok(())
            };

            match &self.source {
                Some(source) => source
                    .clone()
                    .into_iter()
                    .map(|(key, value)| (key.into(), value.into()))
                    .try_for_each(collector),
                None => env::vars_os().try_for_each(collector),
            }?;

            Ok(m)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn collect_builds_nested_tables_from_separator() {
            let mut source = Map::new();
            source.insert("KUMA__URL".into(), "http://localhost:3001".into());
            source.insert("KUMA__TLS__VERIFY".into(), "false".into());

            let values = Environment::default()
                .separator("__")
                .source(Some(source))
                .collect()
                .unwrap();

            let Some(kuma) = values.get("kuma") else {
                panic!("expected top-level kuma table");
            };

            let ValueKind::Table(kuma) = &kuma.kind else {
                panic!("expected kuma to be a table");
            };

            let Some(url) = kuma.get("url") else {
                panic!("expected kuma.url value");
            };
            assert_eq!(url.kind, ValueKind::String("http://localhost:3001".into()));

            let Some(tls) = kuma.get("tls") else {
                panic!("expected kuma.tls table");
            };

            let ValueKind::Table(tls) = &tls.kind else {
                panic!("expected kuma.tls to be a table");
            };

            let Some(verify) = tls.get("verify") else {
                panic!("expected kuma.tls.verify value");
            };
            assert_eq!(verify.kind, ValueKind::String("false".into()));
        }

        #[test]
        fn collect_errors_on_scalar_table_path_conflict() {
            let mut source = Map::new();
            source.insert("KUMA".into(), "http://localhost:3001".into());
            source.insert("KUMA__URL".into(), "http://localhost:3002".into());

            let error = Environment::default()
                .separator("__")
                .source(Some(source))
                .collect()
                .unwrap_err();

            assert!(error.to_string().contains("Conflicting config definition"));
        }

        #[test]
        fn collect_ignores_empty_values_when_flag_set() {
            let mut source = Map::new();
            source.insert("FOO__BAR".into(), "".into());
            source.insert("FOO__BAZ".into(), "present".into());

            let values = Environment::default()
                .separator("__")
                .ignore_empty(true)
                .source(Some(source))
                .collect()
                .unwrap();

            let ValueKind::Table(foo) = &values.get("foo").expect("expected foo").kind else {
                panic!("expected foo table");
            };

            assert!(foo.get("bar").is_none(), "empty value should be ignored");
            assert!(foo.get("baz").is_some(), "non-empty value must be present");
        }

        #[test]
        fn collect_secret_file_replaces_value_with_file_contents() {
            let dir = std::env::temp_dir();
            let path = dir.join("autokuma_test_secret_load.txt");
            std::fs::write(&path, "s3cr3t\n").unwrap();

            let mut source = Map::new();
            source.insert("FOO__PASSWORD".into(), format!("@{}", path.display()));

            let values = Environment::default()
                .separator("__")
                .secret_files(true)
                .source(Some(source))
                .collect()
                .unwrap();

            let ValueKind::Table(foo) = &values.get("foo").expect("expected foo").kind else {
                panic!("expected foo table");
            };
            let pw = foo.get("password").expect("expected password");
            assert_eq!(pw.kind, ValueKind::String("s3cr3t".into()));

            std::fs::remove_file(path).ok();
        }

        #[test]
        fn collect_secret_file_strips_single_trailing_newline() {
            let dir = std::env::temp_dir();
            let path = dir.join("autokuma_test_secret_newline.txt");
            std::fs::write(&path, "value\r\n").unwrap();

            let mut source = Map::new();
            source.insert("KEY".into(), format!("@{}", path.display()));

            let values = Environment::default()
                .secret_files(true)
                .source(Some(source))
                .collect()
                .unwrap();

            let v = values.get("key").expect("expected key");
            assert_eq!(v.kind, ValueKind::String("value".into()));

            std::fs::remove_file(path).ok();
        }

        #[test]
        fn collect_secret_file_escape_preserves_literal_at() {
            let mut source = Map::new();
            source.insert("KEY".into(), "@@/not/a/file".into());

            let values = Environment::default()
                .secret_files(true)
                .source(Some(source))
                .collect()
                .unwrap();

            let v = values.get("key").expect("expected key");
            assert_eq!(v.kind, ValueKind::String("@/not/a/file".into()));
        }

        #[test]
        fn collect_secret_file_missing_returns_error() {
            let mut source = Map::new();
            source.insert("KEY".into(), "@/does/not/exist/autokuma_secret".into());

            let err = Environment::default()
                .secret_files(true)
                .source(Some(source))
                .collect()
                .unwrap_err();

            assert!(err.to_string().contains("failed to read secret file"));
        }

        #[test]
        fn collect_secret_file_disabled_passes_through_at_value() {
            let mut source = Map::new();
            source.insert("KEY".into(), "@/some/path".into());

            let values = Environment::default()
                .secret_files(false)
                .source(Some(source))
                .collect()
                .unwrap();

            let v = values.get("key").expect("expected key");
            assert_eq!(v.kind, ValueKind::String("@/some/path".into()));
        }
    }
}
