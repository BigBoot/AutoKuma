use clap::Parser as _;
use cli::{Cli, Commands};
use flexi_logger::Logger;
use kuma_client::Config;

mod cli;
mod docker_host;
mod login;
mod maintenance;
mod monitor;
mod notification;
mod status_page;
mod tag;
mod utils;

#[tokio::main()]
async fn main() {
    let logger = Logger::try_with_env_or_str("info")
        .unwrap()
        .start()
        .unwrap();

    let cli = Cli::parse();
    let config = Config::from(cli.clone());

    match &cli.command {
        Some(Commands::Monitor { command }) => monitor::handle(command, &config, &cli).await,
        Some(Commands::Notification { command }) => {
            notification::handle(command, &config, &cli).await
        }
        Some(Commands::Tag { command }) => tag::handle(command, &config, &cli).await,
        Some(Commands::Maintenance { command }) => {
            maintenance::handle(command, &config, &cli).await
        }
        Some(Commands::StatusPage { command }) => status_page::handle(command, &config, &cli).await,
        Some(Commands::DockerHost { command }) => docker_host::handle(command, &config, &cli).await,
        Some(Commands::Login { command }) => login::handle(command, &config, &cli).await,
        None if cli.shadow => kuma_client::build::print_build_in(),
        None => {}
    };

    logger.shutdown();
}
