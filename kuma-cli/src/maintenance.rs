use clap::Subcommand;
use kuma_client::Config;
use std::path::PathBuf;
use crate::{cli::Cli, utils::{connect, load_file, PrintResult as _}};

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Command {
    /// Add a new Maintenance
    Add { file: PathBuf },
    /// Edit a Maintenance
    Edit { file: PathBuf },
    /// Get a Maintenance
    Get { id: i32 },
    /// Delete a Maintenance
    Delete { id: i32 },
    /// Get all Maintenances
    List {},
    /// Start/Resume a Maintenance
    Resume { id: i32 },
    /// Stop/Pause a Maintenance
    Pause { id: i32 },
}


pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Add { file }) => connect(config, cli)
            .await
            .add_maintenance(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Edit { file }) => connect(config, cli)
            .await
            .edit_maintenance(load_file(file, cli).await)
            .await
            .print_result(cli),

        Some(Command::Get { id }) => connect(config, cli)
            .await
            .get_maintenance(*id)
            .await
            .print_result(cli),

        Some(Command::Delete { id }) => connect(config, cli)
            .await
            .delete_maintenance(*id)
            .await
            .print_result(cli),

        Some(Command::List {}) => connect(config, cli)
            .await
            .get_maintenances()
            .await
            .print_result(cli),

        Some(Command::Resume { id }) => connect(config, cli)
            .await
            .resume_maintenance(*id)
            .await
            .print_result(cli),

        Some(Command::Pause { id }) => connect(config, cli)
            .await
            .pause_maintenance(*id)
            .await
            .print_result(cli),

        None => {}
    }
}
