use clap::{arg, command, CommandFactory, Parser, Subcommand, ValueEnum};
use kuma_client::{
    build::{
        BRANCH, BUILD_TIME, GIT_CLEAN, LAST_TAG, RUST_CHANNEL, RUST_VERSION, SHORT_COMMIT, TAG,
    },
    Config,
};
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use tokio::task;

type Result<T> = kuma_client::Result<T>;

const VERSION: &str = const_str::format!(
    "{}{}",
    LAST_TAG,
    if const_str::equal!(TAG, "") {
        const_str::format!(
            "-{}{}",
            SHORT_COMMIT,
            if !GIT_CLEAN { "-dirty" } else { "" }
        )
    } else {
        ""
    }
);
const LONG_VERSION: &str = const_str::format!(
    r#"{}
branch: {}
commit_hash: {} 
build_time: {}
build_env: {}, {}"#,
    VERSION,
    BRANCH,
    SHORT_COMMIT,
    BUILD_TIME,
    RUST_VERSION,
    RUST_CHANNEL
);

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Json,
    Toml,
    Yaml,
}

#[derive(Parser, Clone, Debug)]
#[command(author, version = VERSION, long_version = LONG_VERSION, about, long_about = None)]
struct Cli {
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

    #[command(subcommand)]
    command: Option<Commands>,
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
enum Commands {
    /// Manage Monitors
    Monitor {
        #[command(subcommand)]
        command: Option<MonitorCommands>,
    },
    /// Manage Notifications
    Notification {
        #[command(subcommand)]
        command: Option<NotificationCommands>,
    },
    /// Manage Tags
    Tag {
        #[command(subcommand)]
        command: Option<TagCommands>,
    },
}

#[derive(Subcommand, Clone, Debug)]
enum MonitorCommands {
    /// Add a new Monitor
    Add { file: PathBuf },
    /// Edit a Monitor
    Edit { file: PathBuf },
    /// Get a Monitor
    Get { id: i32 },
    /// Delete a Monitor
    Delete { id: i32 },
    /// Get all Monitor
    List {},
}

#[derive(Subcommand, Clone, Debug)]
enum TagCommands {
    /// Add a new Tag
    Add { file: PathBuf },
    /// Edit a Tag
    Edit { file: PathBuf },
    /// Get a Tag
    Get { id: i32 },
    /// Delete a Tag
    Delete { id: i32 },
    /// Get all Tags
    List {},
}

#[derive(Subcommand, Clone, Debug)]
enum NotificationCommands {
    /// Add a new Notification
    Add { file: PathBuf },
    /// Edit a Notification
    Edit { file: PathBuf },
    /// Get a Notification
    Get { id: i32 },
    /// Delete a Notification
    Delete { id: i32 },
    /// Get all Notifications
    List {},
}

trait PrintResult {
    fn print_result(self, cli: &Cli);
}

impl<T> PrintResult for Result<T>
where
    T: Sized + Serialize,
{
    fn print_result(self, cli: &Cli) {
        let value = self.unwrap_or_die(cli);
        print_value(&value, cli);
    }
}

trait ResultOrDie<T> {
    fn unwrap_or_die(self, cli: &Cli) -> T;
}

impl<T, E> ResultOrDie<T> for std::result::Result<T, E>
where
    E: ToString,
{
    fn unwrap_or_die(self, cli: &Cli) -> T {
        match self {
            Ok(t) => t,
            Err(error) => {
                print_value(&json!({"error": error.to_string()}), cli);
                std::process::exit(1)
            }
        }
    }
}

fn print_value<T>(value: &T, cli: &Cli)
where
    T: Serialize,
{
    let str = match (&cli.output_format, &cli.output_pretty) {
        (OutputFormat::Json, true) => serde_json::to_string_pretty(value).unwrap(),
        (OutputFormat::Json, false) => serde_json::to_string(value).unwrap(),
        (OutputFormat::Toml, true) => toml::to_string_pretty(value).unwrap(),
        (OutputFormat::Toml, false) => toml::to_string(value).unwrap(),
        (OutputFormat::Yaml, true) => serde_yaml::to_string(value).unwrap(),
        (OutputFormat::Yaml, false) => serde_yaml::to_string(value).unwrap(),
    };

    print!("{}", str);
}

async fn load_file<T>(file: &PathBuf, cli: &Cli) -> T
where
    T: Send + for<'de> serde::Deserialize<'de> + 'static,
{
    let file_clone = file.clone();
    let cli_clone = cli.clone();

    task::spawn_blocking(move || {
        if file_clone.to_string_lossy() == "-" {
            serde_json::from_reader(std::io::stdin()).unwrap_or_die(&cli_clone)
        } else {
            serde_json::from_reader(std::fs::File::open(&file_clone).unwrap_or_die(&cli_clone))
                .unwrap_or_die(&cli_clone)
        }
    })
    .await
    .unwrap_or_die(cli)
}

async fn monitor_commands(command: &Option<MonitorCommands>, config: &Config, cli: &Cli) {
    match command {
        Some(MonitorCommands::Add { file }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .add_monitor(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(MonitorCommands::Edit { file }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .edit_monitor(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(MonitorCommands::Get { id }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .get_monitor(*id)
            .await
            .print_result(cli),

        Some(MonitorCommands::Delete { id }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .delete_monitor(*id)
            .await
            .print_result(cli),

        Some(MonitorCommands::List {}) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .get_monitors()
            .await
            .print_result(cli),

        None => {}
    }
}

async fn notification_commands(command: &Option<NotificationCommands>, config: &Config, cli: &Cli) {
    match command {
        Some(NotificationCommands::Add { file }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .add_notification(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(NotificationCommands::Edit { file }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .edit_notification(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(NotificationCommands::Get { id }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .get_notification(*id)
            .await
            .print_result(cli),

        Some(NotificationCommands::Delete { id }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .delete_notification(*id)
            .await
            .print_result(cli),

        Some(NotificationCommands::List {}) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .get_notifications()
            .await
            .print_result(cli),

        None => {}
    }
}

async fn tag_commands(command: &Option<TagCommands>, config: &Config, cli: &Cli) {
    match command {
        Some(TagCommands::Add { file }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .add_tag(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(TagCommands::Edit { file }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .edit_tag(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(TagCommands::Get { id }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .get_tag(*id)
            .await
            .print_result(cli),

        Some(TagCommands::Delete { id }) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .delete_tag(*id)
            .await
            .print_result(cli),

        Some(TagCommands::List {}) => kuma_client::Client::connect(config.clone())
            .await
            .unwrap_or_die(cli)
            .get_tags()
            .await
            .print_result(cli),

        None => {}
    }
}

#[tokio::main()]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let cli = Cli::parse();
    let config = Config::from(cli.clone());

    match &cli.command {
        Some(Commands::Monitor { command }) => monitor_commands(command, &config, &cli).await,
        Some(Commands::Notification { command }) => {
            notification_commands(command, &config, &cli).await
        }
        Some(Commands::Tag { command }) => tag_commands(command, &config, &cli).await,
        None => {}
    };
}
