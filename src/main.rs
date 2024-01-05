use std::sync::Arc;

mod config;
mod kuma;
mod sync;
mod util;

#[tokio::main()]
async fn main() {
    let config = Arc::new(
        confique::Config::builder()
            .env()
            .file("config.toml")
            .load()
            .expect("Invalid config"),
    );

    let sync = sync::Sync::new(config);
    sync.run().await;
}
