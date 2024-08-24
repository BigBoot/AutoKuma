use crate::{
    cli::Cli,
    utils::{connect, load_files, CollectOrUnwrap, PrintResult as _},
};
use clap::Subcommand;
use futures_util::{future::join_all, FutureExt};
use kuma_client::{error::Result, Config};
use std::path::PathBuf;
use tap::Pipe;

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Command {
    /// Add a new Maintenance
    Add { file: Vec<PathBuf> },
    /// Edit a Maintenance
    Edit { file: Vec<PathBuf> },
    /// Get a Maintenance
    Get { id: Vec<i32> },
    /// Delete a Maintenance
    Delete { id: Vec<i32> },
    /// Get all Maintenances
    List {},
    /// Start/Resume a Maintenance
    Resume { id: Vec<i32> },
    /// Stop/Pause a Maintenance
    Pause { id: Vec<i32> },
}

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .pipe_borrow(|client| {
                load_files(file, cli).then(|values| {
                    join_all(
                        values
                            .into_iter()
                            .map(|value| client.add_maintenance(value)),
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
                            .map(|value| client.edit_maintenance(value)),
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
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.get_maintenance(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.delete_maintenance(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_maintenances()
            .await
            .print_result(cli),

        Some(Command::Resume { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.resume_maintenance(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Pause { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.pause_maintenance(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        None => {}
    }
}
