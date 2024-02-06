use crate::deserialize::DeserializeVecLenient;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;
use serde_inline_default::serde_inline_default;
use serde_with::{formats::CommaSeparator, serde_as, PickFirst, StringWithSeparator};
use url::Url;

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: Url::parse("http://localhost:3001").unwrap(),
            username: None,
            password: None,
            mfa_token: None,
            headers: Vec::new(),
            connect_timeout: 30.0,
            call_timeout: 30.0,
        }
    }
}
