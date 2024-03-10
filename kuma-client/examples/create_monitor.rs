use kuma_client::{
    monitor::{MonitorGroup, MonitorHttp},
    notification::Notification,
    tag::{Tag, TagDefinition},
    Client, Config, Url,
};

#[tokio::main()]
async fn main() {
    // Connect to the server
    let client = Client::connect(Config {
        url: Url::parse("http://localhost:3001").expect("Invalid URL"),
        username: Some("Username".to_owned()),
        password: Some("Password".to_owned()),
        ..Default::default()
    })
    .await
    .expect("Failed to connect to server");

    // Create a tag
    let tag_definition = client
        .add_tag(TagDefinition {
            name: Some("example_tag".to_owned()),
            color: Some("red".to_owned()),
            ..Default::default()
        })
        .await
        .expect("Failed to add tag");

    // Create a group
    let group = client
        .add_monitor(MonitorGroup {
            name: Some("Example Group".to_owned()),
            tags: vec![Tag {
                tag_id: tag_definition.tag_id,
                value: Some("example_group".to_owned()),
                ..Default::default()
            }],
            ..Default::default()
        })
        .await
        .expect("Failed to add group");

    // Createa a notification
    let notification = client
        .add_notification(Notification {
            name: Some("Example Notification".to_owned()),
            config: Some(serde_json::json!({
                "webhookURL": "https://webhook.site/304eeaf2-0248-49be-8985-2c86175520ca",
                "webhookContentType": "json"
            })),
            ..Default::default()
        })
        .await
        .expect("Failed to add notification");

    // Create a monitor
    client
        .add_monitor(MonitorHttp {
            name: Some("Monitor Name".to_owned()),
            url: Some("https://example.com".to_owned()),
            parent: group.common().id().clone(),
            tags: vec![Tag {
                tag_id: tag_definition.tag_id,
                value: Some("example_monitor".to_owned()),
                ..Default::default()
            }],
            notification_id_list: Some(
                vec![(
                    notification.id.expect("No notification ID").to_string(),
                    true,
                )]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        })
        .await
        .expect("Failed to add monitor");

    let monitors = client.get_monitors().await.expect("Failed to get monitors");
    println!("{:?}", monitors);
}
