use crate::{
    cli::Cli,
    utils::{connect, load_file, PrintResult as _},
};
use clap::Subcommand;
use kuma_client::Config;
use std::path::PathBuf;

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Command {
    /// Add a new StatusPage
    Add { file: PathBuf },
    /// Edit a StatusPage
    Edit { file: PathBuf },
    /// Get a StatusPage
    Get { slug: String },
    /// Delete a StatusPage
    Delete { slug: String },
    /// Get all StatusPages
    List {},
}

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .add_status_page(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .edit_status_page(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Get { slug }) => connect(config, cli)
            .await
            .get_status_page(slug)
            .await
            .print_result(cli),

        Some(Command::Delete { slug }) => connect(config, cli)
            .await
            .delete_status_page(slug)
            .await
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_status_pages()
            .await
            .print_result(cli),

        None => {}
    }
}
