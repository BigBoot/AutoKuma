use crate::deserialize::{DeserializeNumberLenient, DeserializeValueLenient};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub struct Notification {
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "active")]
    pub active: Option<bool>,

    #[serde(rename = "userId")]
    pub user_id: Option<i32>,

    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,

    #[serde(rename = "config")]
    #[serde_as(as = "Option<DeserializeValueLenient>")]
    pub config: Option<serde_json::Value>,
}

pub type NotificationList = Vec<Notification>;
