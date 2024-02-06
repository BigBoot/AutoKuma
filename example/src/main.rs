use kuma_client::{Client, Config, Url};

#[tokio::main()]
async fn main() {
    let client = Client::connect(Config {
        url: Url::parse("http://localhost:3001").expect("Invalid URL"),
        username: Some("Username".to_owned()),
        password: Some("Password".to_owned()),
        ..Default::default()
    })
    .await
    .expect("Failed to connect to server");

    let monitors = client.get_monitors().await.expect("Failed to get monitors");
    println!("{:?}", monitors);
}
