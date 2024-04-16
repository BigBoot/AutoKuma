


use clap::{command, Subcommand};
use kuma_client::Config;
use std::path::PathBuf;
use crate::{cli::Cli, utils::{connect, load_file, PrintResult as _}};

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)] 
pub(crate) enum Command {
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

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .add_tag(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .edit_tag(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Get { id }) => connect(config, cli)
            .await
            .get_tag(*id)
            .await
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .delete_tag(*id)
            .await
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_tags()
            .await
            .print_result(cli),

        None => {}
    }
}