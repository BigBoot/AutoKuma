//! Models related to Uptime Kuma notification services

use crate::deserialize::{
    DeserializeBoolLenient, DeserializeNumberLenient, DeserializeValueLenient,
};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

const IGNORE_ATTRIBUTES: [&str; 6] = ["isDefault", "id", "active", "user_id", "config", "name"];

/// Represents a notification service in Uptime Kuma.
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Default, Debug, Serialize, Derivative, Deserialize, Eq)]
#[derivative(PartialEq)]
pub struct Notification {
    /// The unique identifier for the notification service.
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    /// The name of the notification.
    #[serde(rename = "name")]
    pub name: Option<String>,

    /// Indicates whether the notification service is active or not.
    #[serde(rename = "active")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub active: Option<bool>,

    /// The user identifier associated with the notification service.
    #[serde(rename = "user_id")]
    #[serde(alias = "user_id")]
    pub user_id: Option<i32>,

    /// Indicates whether the notification service is enabled by default.
    #[serde(rename = "isDefault")]
    #[serde(alias = "is_default")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub is_default: Option<bool>,

    /// Additional service specific configuration in JSON format.
    #[serde(rename = "config")]
    #[serde_as(as = "Option<DeserializeValueLenient>")]
    #[derivative(PartialEq(compare_with = "config_eq"))]
    pub config: Option<serde_json::Value>,
}

fn config_eq(a: &Option<serde_json::Value>, b: &Option<serde_json::Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(serde_json::Value::Object(map_a)), Some(serde_json::Value::Object(map_b))) => {
            let count_a = map_a
                .iter()
                .filter(|(k, _)| !IGNORE_ATTRIBUTES.contains(&k.as_str()))
                .count();

            let count_b = map_b
                .iter()
                .filter(|(k, _)| !IGNORE_ATTRIBUTES.contains(&k.as_str()))
                .count();

            if count_a != count_b {
                return false;
            }

            map_a
                .iter()
                .filter(|(k, _)| !IGNORE_ATTRIBUTES.contains(&k.as_str()))
                .all(|(k, v)| map_b.get(k).map_or(false, |v_b| v == v_b))
        }
        _ => a == b,
    }
}

/// A list of notification services.
pub type NotificationList = Vec<Notification>;
