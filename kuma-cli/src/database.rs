use crate::{
    cli::Cli,
    utils::{connect, PrintResult as _},
};
use clap::Subcommand;
use kuma_client::Config;
use serde_json::json;

#[derive(Subcommand, Clone, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum Command {
    /// Get the size of the statistics database
    Size,
    /// Shrink (vacuum) the statistics database
    Shrink,
}

pub(crate) async fn handle(command: &Option<Command>, config: &Config, cli: &Cli) {
    match command {
        Some(Command::Size) => connect(config, cli)
            .await
            .get_database_size()
            .await
            .map(|size| json!({"size": size}))
            .print_result(cli),

        Some(Command::Shrink) => connect(config, cli)
            .await
            .shrink_database()
            .await
            .print_result(cli),

        None => {}
    }
}
