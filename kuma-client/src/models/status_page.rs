//! Models related to Uptime Kuma status pages

use crate::{
    deserialize::{DeserializeBoolLenient, DeserializeNumberLenient, DeserializeVecLenient},
    monitor::MonitorType,
};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use serde_with::{serde_as, skip_serializing_none};
use std::collections::HashMap;

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, Derivative, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct PublicGroupMonitor {
    #[serde(rename = "id")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    #[serde(rename = "name")]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub name: Option<String>,

    #[serde(skip_serializing)]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub entity_id: Option<String>,

    #[serde(rename = "weight")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub weight: Option<bool>,

    #[serde(rename = "type")]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub monitor_type: Option<MonitorType>,
}
crate::default_from_serde!(PublicGroupMonitor);

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, Derivative, Serialize, Deserialize)]
#[derivative(PartialEq)]
pub struct PublicGroup {
    #[serde(rename = "id")]
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub id: Option<i32>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "weight")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub weight: Option<i32>,

    #[serde(rename = "monitorList", alias = "monitor_list", default)]
    pub monitor_list: PublicGroupMonitorList,
}
crate::default_from_serde!(PublicGroup);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnalyticsType {
    #[serde(rename = "google")]
    Google,
    #[serde(rename = "umami")]
    Umami,
    #[serde(rename = "plausible")]
    Plausible,
    #[serde(rename = "matomo")]
    Matomo,
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
    #[serde_inline_default(Some("default".to_owned()))]
    pub slug: Option<String>,

    #[serde(rename = "title")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "icon")]
    #[serde_inline_default(Some("/icon.svg".to_owned()))]
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
    #[serde_as(as = "DeserializeVecLenient<String>")]
    pub domain_name_list: Vec<String>,

    #[serde(rename = "customCSS")]
    #[serde_inline_default(Some("body {\n  \n}\n".to_owned()))]
    pub custom_css: Option<String>,

    #[serde(rename = "footerText")]
    pub footer_text: Option<String>,

    #[serde(rename = "showPoweredBy")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub show_powered_by: Option<bool>,

    #[serde(rename = "analyticsType")]
    #[serialize_always]
    pub analytics_type: Option<AnalyticsType>,

    #[serde(rename = "analyticsId")]
    pub analytics_id: Option<String>,

    #[serde(rename = "analyticsScriptUrl")]
    pub analytics_script_url: Option<String>,

    #[serde(rename = "showCertificateExpiry")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub show_certificate_expiry: Option<bool>,

    #[serde(rename = "publicGroupList", alias = "public_group_list")]
    #[serde_as(as = "Option<DeserializeVecLenient<PublicGroup>>")]
    pub public_group_list: Option<PublicGroupList>,
}
crate::default_from_serde!(StatusPage);

pub type StatusPageList = HashMap<String, StatusPage>;
pub type PublicGroupList = Vec<PublicGroup>;
pub type PublicGroupMonitorList = Vec<PublicGroupMonitor>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_page_deserializes_snake_case_aliases_and_entity_id() {
        let json = serde_json::json!({
            "type": "status_page",
            "slug": "production",
            "title": "Production Status",
            "public_group_list": [
                {
                    "name": "Services",
                    "monitor_list": [
                        { "entity_id": "my-app-kuma" },
                        { "entity_id": "my-db-kuma" },
                    ]
                }
            ]
        });

        let page: StatusPage = serde_json::from_value(json).expect("should parse status page");
        let group = page.public_group_list.expect("should have groups").into_iter().next().expect("should have one group");
        assert_eq!(group.name.as_deref(), Some("Services"));
        assert_eq!(group.monitor_list.len(), 2);
        assert_eq!(group.monitor_list[0].entity_id.as_deref(), Some("my-app-kuma"));
        assert!(group.monitor_list[0].id.is_none());
    }

    #[test]
    fn public_group_monitor_skips_entity_id_on_serialization() {
        let monitor = PublicGroupMonitor {
            id: Some(42),
            name: None,
            entity_id: Some("my-app-kuma".to_owned()),
            weight: None,
            monitor_type: None,
        };

        let json = serde_json::to_value(&monitor).expect("should serialize");
        assert!(json.get("entity_id").is_none());
        assert_eq!(json.get("id").and_then(|v| v.as_i64()), Some(42));
    }
}
