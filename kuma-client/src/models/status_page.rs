//! Models related to Uptime Kuma status pages

use crate::{
    deserialize::{DeserializeBoolLenient, DeserializeNumberLenient},
    monitor::MonitorType,
};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use serde_with::{serde_as, skip_serializing_none};
use std::collections::HashMap;

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicGroupMonitor {
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "weight")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub weight: Option<bool>,

    #[serde(rename = "type")]
    pub monitor_type: Option<MonitorType>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicGroup {
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "weight")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub weight: Option<i32>,

    #[serde(rename = "monitorList", default)]
    pub monitor_list: PublicGroupMonitorList,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatusPage {
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    #[serde(rename = "slug")]
    pub slug: Option<String>,

    #[serde(rename = "title")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "icon")]
    pub icon: Option<String>,

    #[serde(rename = "theme")]
    pub theme: Option<String>,

    #[serde(rename = "published")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub published: Option<bool>,

    #[serde(rename = "showTags")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub show_tags: Option<bool>,

    #[serde(rename = "domainNameList", default)]
    pub domain_name_list: Vec<String>,

    #[serde(rename = "customCSS")]
    pub custom_css: Option<String>,

    #[serde(rename = "footerText")]
    pub footer_text: Option<String>,

    #[serde(rename = "showPoweredBy")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub show_powered_by: Option<bool>,

    #[serde(rename = "googleAnalyticsId")]
    pub google_analytics_id: Option<String>,

    #[serde(rename = "showCertificateExpiry")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub show_certificate_expiry: Option<bool>,

    #[serde(rename = "publicGroupList")]
    pub public_group_list: Option<PublicGroupList>,
}

pub type StatusPageList = HashMap<String, StatusPage>;
pub type PublicGroupList = Vec<PublicGroup>;
pub type PublicGroupMonitorList = Vec<PublicGroupMonitor>;
