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

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .add_notification(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .edit_notification(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Get { id }) => connect(config, cli)
            .await
            .get_notification(*id)
            .await
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .delete_notification(*id)
            .await
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_notifications()
            .await
            .print_result(cli),

        None => {}
    }
}
