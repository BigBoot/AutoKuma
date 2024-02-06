//! Models related to Uptime Kuma tags

use crate::deserialize::DeserializeNumberLenient;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct TagDefinition {
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub tag_id: Option<i32>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "color")]
    pub color: Option<String>,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub struct Tag {
    #[serde(rename = "tag_id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub tag_id: Option<i32>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "color")]
    pub color: Option<String>,

    #[serde(rename = "value")]
    pub value: Option<String>,
}

impl From<TagDefinition> for Tag {
    fn from(value: TagDefinition) -> Self {
        Tag {
            name: value.name,
            color: value.color,
            tag_id: value.tag_id,
            value: None,
        }
    }
}

impl From<Tag> for TagDefinition {
    fn from(value: Tag) -> Self {
        TagDefinition {
            tag_id: value.tag_id,
            name: value.name,
            color: value.color,
        }
    }
}
