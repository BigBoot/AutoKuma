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
            tls: TlsConfig::default(),
        }
    }
}
