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
    /// Add a new Tag
    Add { file: Vec<PathBuf> },
    /// Edit a Tag
    Edit { file: Vec<PathBuf> },
    /// Get a Tag
    Get { id: Vec<i32> },
    /// Delete a Tag
    Delete { id: Vec<i32> },
    /// Get all Tags
    List {},
}

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .pipe_borrow(|client| {
                load_files(file, cli)
                    .then(|values| join_all(values.into_iter().map(|value| client.add_tag(value))))
            })
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .pipe_borrow(|client| {
                load_files(file, cli)
                    .then(|values| join_all(values.into_iter().map(|value| client.edit_tag(value))))
            })
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Get { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.get_tag(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(id.iter().map(|id| client.delete_tag(*id))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_tags()
            .await
            .print_result(cli),

        None => {}
    }
}
