use crate::{
    cli::Cli,
    utils::{connect, load_file, PrintResult as _},
};
use clap::Subcommand;
use kuma_client::{monitor::Monitor, Config};
use std::path::PathBuf;

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Command {
    /// Add a new Monitor
    Add { file: PathBuf },
    /// Edit a Monitor
    Edit { file: PathBuf },
    /// Get a Monitor
    Get { id: i32 },
    /// Delete a Monitor
    Delete { id: i32 },
    /// Get all Monitors
    List {},
    /// Start/Resume a Monitor
    Resume { id: i32 },
    /// Stop/Pause a Monitor
    Pause { id: i32 },
}

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .add_monitor(load_file::<Monitor>(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .edit_monitor(load_file::<Monitor>(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Get { id }) => connect(config, cli)
            .await
            .get_monitor(*id)
            .await
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .delete_monitor(*id)
            .await
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_monitors()
            .await
            .print_result(cli),

        Some(Command::Resume { id }) => connect(config, cli)
            .await
            .resume_monitor(*id)
            .await
            .print_result(cli),

        Some(Command::Pause { id }) => connect(config, cli)
            .await
            .pause_monitor(*id)
            .await
            .print_result(cli),

        None => {}
    }
}
