use crate::{
    cli::Cli,
    utils::{connect, load_file, PrintResult as _},
};
use clap::Subcommand;
use kuma_client::Config;
use serde_json::json;
use std::path::PathBuf;

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Command {
    /// Add a new DockerHost
    Add { file: PathBuf },
    /// Edit a DockerHost
    Edit { file: PathBuf },
    /// Get a DockerHost
    Get { id: i32 },
    /// Delete a DockerHost
    Delete { id: i32 },
    /// Get all DockerHosts
    List {},
    /// Test a DockerHost config
    Test { file: PathBuf },
}

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .add_docker_host(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .edit_docker_host(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Get { id }) => connect(config, cli)
            .await
            .get_docker_host(*id)
            .await
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .delete_docker_host(*id)
            .await
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_docker_hosts()
            .await
            .print_result(cli),

        Some(Command::Test { file }) => connect(config, cli)
            .await
            .test_docker_host(&load_file(file, cli).await)
            .await
            .map(|response| json!({"msg": response}))
            .print_result(cli),

        None => {}
    }
}
