//! Models related to Uptime Kuma notification services

use crate::deserialize::{DeserializeNumberLenient, DeserializeValueLenient};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

/// Represents a notification service in Uptime Kuma.
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
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
    pub active: Option<bool>,

    /// The user identifier associated with the notification service.
    #[serde(rename = "userId")]
    pub user_id: Option<i32>,

    /// Indicates whether the notification service is enabled by default.
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,

    /// Additional service specific configuration in JSON format.
    #[serde(rename = "config")]
    #[serde_as(as = "Option<DeserializeValueLenient>")]
    pub config: Option<serde_json::Value>,
}

/// A list of notification services.
pub type NotificationList = Vec<Notification>;
