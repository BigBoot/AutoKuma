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

    /// The username for logging into Uptime Kuma (required unless auth is disabled).
    pub username: Option<String>,

    /// The password for logging into Uptime Kuma (required unless auth is disabled).
    /// Can be set via AUTOKUMA__KUMA__PASSWORD or AUTOKUMA__KUMA__PASSWORD_FILE.
    pub password: Option<String>,

    /// The MFA token for logging into Uptime Kuma (required if MFA is enabled).
    /// Can be set via AUTOKUMA__KUMA__MFA_TOKEN or AUTOKUMA__KUMA__MFA_TOKEN_FILE.
    pub mfa_token: Option<String>,

    /// The MFA secret. Used to generate tokens for logging into Uptime Kuma (alternative to a single_use mfa_token).
    /// Can be set via AUTOKUMA__KUMA__MFA_SECRET or AUTOKUMA__KUMA__MFA_SECRET_FILE.
    pub mfa_secret: Option<String>,

    /// JWT Auth token received after a successful login, can be used as an alternative to username/password.
    /// Can be set via AUTOKUMA__KUMA__AUTH_TOKEN or AUTOKUMA__KUMA__AUTH_TOKEN_FILE.
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

impl Config {
    /// Resolves secret fields from either direct values or _FILE env vars.
    /// If both the direct value and _FILE variant are set, returns an error.
    /// Trims trailing newlines from file contents.
    pub fn resolve_secrets() -> std::result::Result<(), String> {
        Self::resolve_secret_env("AUTOKUMA__KUMA__PASSWORD", "AUTOKUMA__KUMA__PASSWORD_FILE")?;
        Self::resolve_secret_env("AUTOKUMA__KUMA__MFA_TOKEN", "AUTOKUMA__KUMA__MFA_TOKEN_FILE")?;
        Self::resolve_secret_env("AUTOKUMA__KUMA__MFA_SECRET", "AUTOKUMA__KUMA__MFA_SECRET_FILE")?;
        Self::resolve_secret_env("AUTOKUMA__KUMA__AUTH_TOKEN", "AUTOKUMA__KUMA__AUTH_TOKEN_FILE")?;
        Ok(())
    }

    fn resolve_secret_env(var_name: &str, var_file_name: &str) -> std::result::Result<(), String> {
        let has_var = std::env::var(var_name).is_ok();
        let has_file = std::env::var(var_file_name).is_ok();

        if has_var && has_file {
            return Err(format!(
                "both {} and {} are set; choose one",
                var_name, var_file_name
            ));
        }

        if has_file {
            let file_path = std::env::var(var_file_name)
                .map_err(|_| format!("failed to read {}", var_file_name))?;
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| format!("failed to read file {}: {}", file_path, e))?;
            let trimmed = content.trim_end().to_string();
            std::env::set_var(var_name, trimmed);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_secret_from_file() {
        let temp_file = std::env::temp_dir().join("test_secret.txt");
        std::fs::write(&temp_file, "secret_value\n").unwrap();

        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD");
        std::env::set_var("AUTOKUMA__KUMA__PASSWORD_FILE", &temp_file);

        let result = Config::resolve_secret_env(
            "AUTOKUMA__KUMA__PASSWORD",
            "AUTOKUMA__KUMA__PASSWORD_FILE",
        );

        assert!(result.is_ok());
        assert_eq!(
            std::env::var("AUTOKUMA__KUMA__PASSWORD").unwrap(),
            "secret_value"
        );

        std::fs::remove_file(&temp_file).unwrap();
        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD");
        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD_FILE");
    }

    #[test]
    fn test_resolve_secret_strips_newline() {
        let temp_file = std::env::temp_dir().join("test_secret_newline.txt");
        std::fs::write(&temp_file, "value_with_newline\n").unwrap();

        std::env::remove_var("AUTOKUMA__KUMA__MFA_TOKEN");
        std::env::set_var("AUTOKUMA__KUMA__MFA_TOKEN_FILE", &temp_file);

        let result = Config::resolve_secret_env(
            "AUTOKUMA__KUMA__MFA_TOKEN",
            "AUTOKUMA__KUMA__MFA_TOKEN_FILE",
        );

        assert!(result.is_ok());
        assert_eq!(
            std::env::var("AUTOKUMA__KUMA__MFA_TOKEN").unwrap(),
            "value_with_newline"
        );
        assert!(!std::env::var("AUTOKUMA__KUMA__MFA_TOKEN")
            .unwrap()
            .ends_with('\n'));

        std::fs::remove_file(&temp_file).unwrap();
        std::env::remove_var("AUTOKUMA__KUMA__MFA_TOKEN");
        std::env::remove_var("AUTOKUMA__KUMA__MFA_TOKEN_FILE");
    }

    #[test]
    fn test_conflict_both_set() {
        std::env::set_var("AUTOKUMA__KUMA__PASSWORD", "direct_value");
        std::env::set_var("AUTOKUMA__KUMA__PASSWORD_FILE", "/tmp/some_file");

        let result = Config::resolve_secret_env(
            "AUTOKUMA__KUMA__PASSWORD",
            "AUTOKUMA__KUMA__PASSWORD_FILE",
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("both AUTOKUMA__KUMA__PASSWORD and AUTOKUMA__KUMA__PASSWORD_FILE are set"));

        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD");
        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD_FILE");
    }

    #[test]
    fn test_file_not_found() {
        std::env::remove_var("AUTOKUMA__KUMA__AUTH_TOKEN");
        std::env::set_var("AUTOKUMA__KUMA__AUTH_TOKEN_FILE", "/nonexistent/file");

        let result = Config::resolve_secret_env(
            "AUTOKUMA__KUMA__AUTH_TOKEN",
            "AUTOKUMA__KUMA__AUTH_TOKEN_FILE",
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to read file"));

        std::env::remove_var("AUTOKUMA__KUMA__AUTH_TOKEN_FILE");
    }

    #[test]
    fn test_resolve_all_secrets() {
        let temp_dir = std::env::temp_dir();
        let password_file = temp_dir.join("password.txt");
        let mfa_token_file = temp_dir.join("mfa_token.txt");
        let mfa_secret_file = temp_dir.join("mfa_secret.txt");
        let auth_token_file = temp_dir.join("auth_token.txt");

        std::fs::write(&password_file, "password123\n").unwrap();
        std::fs::write(&mfa_token_file, "mfa123\n").unwrap();
        std::fs::write(&mfa_secret_file, "secret456\n").unwrap();
        std::fs::write(&auth_token_file, "token789\n").unwrap();

        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD");
        std::env::remove_var("AUTOKUMA__KUMA__MFA_TOKEN");
        std::env::remove_var("AUTOKUMA__KUMA__MFA_SECRET");
        std::env::remove_var("AUTOKUMA__KUMA__AUTH_TOKEN");

        std::env::set_var("AUTOKUMA__KUMA__PASSWORD_FILE", &password_file);
        std::env::set_var("AUTOKUMA__KUMA__MFA_TOKEN_FILE", &mfa_token_file);
        std::env::set_var("AUTOKUMA__KUMA__MFA_SECRET_FILE", &mfa_secret_file);
        std::env::set_var("AUTOKUMA__KUMA__AUTH_TOKEN_FILE", &auth_token_file);

        let result = Config::resolve_secrets();

        assert!(result.is_ok());
        assert_eq!(
            std::env::var("AUTOKUMA__KUMA__PASSWORD").unwrap(),
            "password123"
        );
        assert_eq!(
            std::env::var("AUTOKUMA__KUMA__MFA_TOKEN").unwrap(),
            "mfa123"
        );
        assert_eq!(
            std::env::var("AUTOKUMA__KUMA__MFA_SECRET").unwrap(),
            "secret456"
        );
        assert_eq!(
            std::env::var("AUTOKUMA__KUMA__AUTH_TOKEN").unwrap(),
            "token789"
        );

        std::fs::remove_file(&password_file).unwrap();
        std::fs::remove_file(&mfa_token_file).unwrap();
        std::fs::remove_file(&mfa_secret_file).unwrap();
        std::fs::remove_file(&auth_token_file).unwrap();

        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD");
        std::env::remove_var("AUTOKUMA__KUMA__PASSWORD_FILE");
        std::env::remove_var("AUTOKUMA__KUMA__MFA_TOKEN");
        std::env::remove_var("AUTOKUMA__KUMA__MFA_TOKEN_FILE");
        std::env::remove_var("AUTOKUMA__KUMA__MFA_SECRET");
        std::env::remove_var("AUTOKUMA__KUMA__MFA_SECRET_FILE");
        std::env::remove_var("AUTOKUMA__KUMA__AUTH_TOKEN");
        std::env::remove_var("AUTOKUMA__KUMA__AUTH_TOKEN_FILE");
    }
}
