use clap::{arg, command, CommandFactory, Parser, Subcommand};
use kuma_client::{
    build::{
        LONG_VERSION, SHORT_VERSION
    }, Config
};

use crate::utils::{OutputFormat, ResultOrDie as _};

#[derive(Parser, Clone, Debug)]
#[command(author, version = SHORT_VERSION, long_version = LONG_VERSION, about, long_about = None, arg_required_else_help = true)]
pub(crate) struct Cli {
    /// The URL AutoKuma should use to connect to Uptime Kuma.
    #[arg(long, global = true)]
    pub url: Option<String>,

    /// The username for logging into Uptime Kuma (required unless auth is disabled).
    #[arg(long, global = true)]
    pub username: Option<String>,

    /// The password for logging into Uptime Kuma (required unless auth is disabled).
    #[arg(long, global = true)]
    pub password: Option<String>,

    /// The MFA token for logging into Uptime Kuma (required if MFA is enabled).
    #[arg(long, global = true)]
    pub mfa_token: Option<String>,

    /// Add a HTTP header when connecting to Uptime Kuma.
    #[arg(long = "header", value_name = "KEY=VALUE", global = true)]
    pub headers: Vec<String>,

    /// The timeout for the initial connection to Uptime Kuma.
    #[arg(long, default_value = "30.0", global = true)]
    pub connect_timeout: Option<f64>,

    /// The timeout for executing calls to the Uptime Kuma server.
    #[arg(long, default_value = "30.0", global = true)]
    pub call_timeout: Option<f64>,

    /// The output format
    #[arg(value_enum, long = "format", default_value_t = OutputFormat::Json, global = true)]
    pub output_format: OutputFormat,

    /// Wether the output should be pretty printed or condensed
    #[arg(long = "pretty", default_value_t = false, global = true)]
    pub output_pretty: bool,

    #[arg(long, hide = true)]
    pub shadow: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl From<Cli> for Config {
    fn from(value: Cli) -> Self {
        config::Config::builder() 
            .add_source(config::File::with_name(&dirs::config_local_dir().map(|dir| dir.join("kuma").join("config").to_string_lossy().to_string()).unwrap_or_default()).required(false))
            .add_source(config::File::with_name("kuma").required(false))
            .add_source(
                config::Environment::with_prefix("KUMA")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .set_default("headers", Vec::<String>::new()).unwrap()
            .set_override_option("url", value.url.clone())
            .unwrap()
            .set_override_option("username", value.username.clone())
            .unwrap()
            .set_override_option("password", value.password.clone())
            .unwrap()
            .set_override_option("mfa_token", value.mfa_token.clone())
            .unwrap()
            .set_override_option(
                "headers",
                match value.headers.is_empty() {
                    true => None,
                    false => Some(value.headers.clone()),
                },
            )
            .unwrap()
            .set_override_option("connect_timeout", value.connect_timeout)
            .unwrap()
            .set_override_option("call_timeout", value.call_timeout)
            .unwrap()
            .build()
            .and_then(|config| config.try_deserialize())
            .unwrap_or_else(|e| match &e {
                config::ConfigError::Message(msg) if msg == "missing field `url`" => Cli::command().error(clap::error::ErrorKind::MissingRequiredArgument, "the following required arguments were not provided:\n  \x1b[32m--url <URL>\x1b[0m").exit(),
                e => Err(e).unwrap_or_die(&value),
            })
    }
}

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Commands {
    /// Manage Monitors
    Monitor {
        #[command(subcommand)]
        command: Option<crate::monitor::Command>,
    },
    /// Manage Notifications
    Notification {
        #[command(subcommand)]
        command: Option<crate::notification::Command>,
    },
    /// Manage Tags
    Tag {
        #[command(subcommand)]
        command: Option<crate::tag::Command>,
    },
    /// Manage Maintenances
    Maintenance {
        #[command(subcommand)]
        command: Option<crate::maintenance::Command>,
    },
    /// Manage Status Pages
    StatusPage {
        #[command(subcommand)]
        command: Option<crate::status_page::Command>, 
    },
    /// Manage Docker Hosts
    DockerHost {
        #[command(subcommand)]
        command: Option<crate::docker_host::Command>,
    },
}