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
    /// Add a new StatusPage
    Add { file: Vec<PathBuf> },
    /// Edit a StatusPage
    Edit { file: Vec<PathBuf> },
    /// Get a StatusPage
    Get { slug: Vec<String> },
    /// Delete a StatusPage
    Delete { slug: Vec<String> },
    /// Get all StatusPages
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
                            .map(|value| client.add_status_page(value)),
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
                            .map(|value| client.edit_status_page(value)),
                    )
                })
            })
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Get { slug }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(slug.iter().map(|slug| client.get_status_page(slug))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::Delete { slug }) => connect(config, cli)
            .await
            .pipe_borrow(|client| join_all(slug.iter().map(|slug| client.delete_status_page(slug))))
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()
            .map(|result| result.into_iter().collect_or_unwrap())
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_status_pages()
            .await
            .print_result(cli),

        None => {}
    }
}
