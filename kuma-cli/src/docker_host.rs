use crate::{
    cli::Cli,
    utils::{connect, load_files, CollectOrUnwrap, PrintResult as _},
};
use clap::Subcommand;
use futures_util::future::{join_all, FutureExt};
use kuma_client::{docker_host::DockerHost, error::Result, Config};
use serde_json::json;
use std::path::PathBuf;
use tap::Pipe;

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Command {
    /// Add a new DockerHost
    Add { file: Vec<PathBuf> },
    /// Edit a DockerHost
    Edit { file: Vec<PathBuf> },
    /// Get a DockerHost
    Get { id: Vec<i32> },
    /// Delete a DockerHost
    Delete { id: Vec<i32> },
    /// Get all DockerHosts
    List {},
    /// Test a DockerHost config
    Test { file: Vec<PathBuf> },
}

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .pipe_borrow(|client| {
                load_files::<DockerHost>(file, cli).then(|values| {
                    join_all(
                        values
                            .iter()
                            .map(|value| client.add_docker_host(value.clone())),
                    )
                })
            })
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .pipe_borrow(|client| {
                load_files(file, cli).then(|values| {
                    join_all(
                        values
                            .into_iter()
                            .map(|value| client.edit_docker_host(value)),
                    )
                })
            })
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Get { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.get_docker_host(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.delete_docker_host(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_docker_hosts()
            .await
            .print_result(cli),

        Some(Command::Test { file }) => connect(config, cli)
            .await
            .pipe_borrow(|client| {
                load_files::<DockerHost>(file, cli).then(|values| {
                    join_all(
                        values
                            .into_iter()
                            .map(|value| client.test_docker_host(value)),
                    )
                })
            })
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|responses| {
                responses
                    .into_iter()
                    .map(|response| json!({"msg": response}))
                    .collect_or_unwrap()
            })
            .print_result(cli),

        None => {}
    }
}
