use crate::{
    cli::Cli,
    utils::{self, print_value},
};
use clap::{CommandFactory, Parser};
use kuma_client::Config;
use serde_json::json;

#[derive(Parser, Clone, Debug)]
#[command()]
pub(crate) struct Command {
    /// The username for logging in
    username: Option<String>,

    /// The password for logging in
    password: Option<String>,

    /// Clear any stored auth token
    #[arg(long)]
    clear: bool,
}

pub(crate) async fn handle(command: &Command, config: &Config, cli: &Cli) {
    if command.clear {
        utils::clear_auth_token().await;
        print_value(&json!({"ok": true, "message" : "auth token cleared"}), cli);
        return;
    }

    if command.username.is_none() {
        Cli::command()
            .error(
                clap::error::ErrorKind::MissingRequiredArgument,
                "the following required arguments were not provided:\n  \x1b[32m<USERNAME|--clear>\x1b[0m",
            )
            .exit();
    }

    let username = command.username.clone().unwrap();
    let password = command
        .password
        .clone()
        .unwrap_or_else(|| rpassword::prompt_password("Password: ").unwrap());

    let config = Config {
        username: Some(username),
        password: Some(password),
        auth_token: None,
        ..config.clone()
    };

    let client = utils::connect(&config, cli).await;

    let auth_token = client.get_auth_token().await;

    if let Some(token) = auth_token {
        print_value(
            &json!({"ok": true, "message" : "login ok", "token": token}),
            cli,
        );
    } else {
        print_value(&json!({"error" : "no auth token received"}), cli);
    }
}
