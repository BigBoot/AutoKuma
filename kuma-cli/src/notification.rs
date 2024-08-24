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
    /// Add a new Notification
    Add { file: Vec<PathBuf> },
    /// Edit a Notification
    Edit { file: Vec<PathBuf> },
    /// Get a Notification
    Get { id: Vec<i32> },
    /// Delete a Notification
    Delete { id: Vec<i32> },
    /// Get all Notifications
    List {},
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
                            .map(|value| client.add_notification(value)),
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
                            .map(|value| client.edit_notification(value)),
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
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.get_notification(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.delete_notification(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_notifications()
            .await
            .print_result(cli),

        None => {}
    }
}
