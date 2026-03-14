//! Models related to Uptime Kuma monitors

use crate::{
    deserialize::{
        DeserializeBoolLenient, DeserializeHashMapLenient, DeserializeNumberLenient,
        DeserializeVecLenient,
    },
    error::{Error, Result},
    models::tag::Tag,
};
use derivative::Derivative;
use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};
use serde_inline_default::serde_inline_default;
use serde_with::{serde_as, skip_serializing_none};
use std::collections::{HashMap, HashSet};

pub trait MonitorCommon {
    fn id(&self) -> &Option<i32>;
    fn id_mut(&mut self) -> &mut Option<i32>;
    fn name(&self) -> &Option<String>;
    fn name_mut(&mut self) -> &mut Option<String>;
    fn description(&self) -> &Option<String>;
    fn description_mut(&mut self) -> &mut Option<String>;
    fn interval(&self) -> &Option<i32>;
    fn interval_mut(&mut self) -> &mut Option<i32>;
    fn active(&self) -> &Option<bool>;
    fn active_mut(&mut self) -> &mut Option<bool>;
    fn max_retries(&self) -> &Option<i32>;
    fn max_retries_mut(&mut self) -> &mut Option<i32>;
    fn retry_interval(&self) -> &Option<i32>;
    fn retry_interval_mut(&mut self) -> &mut Option<i32>;
    fn upside_down(&self) -> &Option<bool>;
    fn upside_down_mut(&mut self) -> &mut Option<bool>;
    fn parent(&self) -> &Option<i32>;
    fn parent_mut(&mut self) -> &mut Option<i32>;
    fn tags(&self) -> &Vec<Tag>;
    fn tags_mut(&mut self) -> &mut Vec<Tag>;
    fn notification_id_list(&self) -> &Option<HashMap<String, bool>>;
    fn notification_id_list_mut(&mut self) -> &mut Option<HashMap<String, bool>>;
    fn accepted_statuscodes(&self) -> &Vec<String>;
    fn accepted_statuscodes_mut(&mut self) -> &mut Vec<String>;
    fn resend_interval(&self) -> &Option<i32>;
    fn resend_interval_mut(&mut self) -> &mut Option<i32>;
    #[cfg(not(feature = "uptime-kuma-v1"))]
    fn conditions(&self) -> &Vec<MonitorCondition>;
    #[cfg(not(feature = "uptime-kuma-v1"))]
    fn conditions_mut(&mut self) -> &mut Vec<MonitorCondition>;

    #[cfg(feature = "private-api")]
    fn parent_name(&self) -> &Option<String>;
    #[cfg(feature = "private-api")]
    fn parent_name_mut(&mut self) -> &mut Option<String>;
    #[cfg(feature = "private-api")]
    fn create_paused(&self) -> &Option<bool>;
    #[cfg(feature = "private-api")]
    fn create_paused_mut(&mut self) -> &mut Option<bool>;
    #[cfg(feature = "private-api")]
    fn notification_names(&self) -> &Option<Vec<String>>;
    #[cfg(feature = "private-api")]
    fn notification_names_mut(&mut self) -> &mut Option<Vec<String>>;
    #[cfg(feature = "private-api")]
    fn tag_names(&self) -> &Option<Vec<super::tag::TagValue>>;
    #[cfg(feature = "private-api")]
    fn tag_names_mut(&mut self) -> &mut Option<Vec<super::tag::TagValue>>;
}

/// Invokes a callback macro in item position.
macro_rules! invoke_monitor_fields_item {
    ($callback:ident, $($args:tt)*) => {
        $callback! {
            $($args)*
        }
    };
}

/// Invokes a callback macro in expression position.
macro_rules! invoke_monitor_fields_expr {
    ($callback:ident, $($args:tt)*) => {
        $callback!(
            $($args)*
        )
    };
}

/// Expands the single source of truth for fields shared by all monitor structs.
///
/// Each entry carries field attributes, cfg-only attributes, the field name, and
/// the field type so the same metadata can drive struct generation and trait impls
macro_rules! with_monitor_common_fields_impl {
    ($invoker:ident, $callback:ident, $($args:tt)*) => {
        $invoker! {
            $callback,
            $($args)*
            [
                (
                    {
                        #[serde(rename = "id")]
                        #[serde_as(as = "Option<DeserializeNumberLenient>")]
                    }
                    {}
                    id, Option<i32>
                ),
                (
                    {
                        #[serde(rename = "name")]
                    }
                    {}
                    name, Option<String>
                ),
                (
                    {
                        #[serde(rename = "description")]
                    }
                    {}
                    description, Option<String>
                ),
                (
                    {
                        #[serde(rename = "interval")]
                        #[serde_inline_default(Some(60))]
                        #[serde_as(as = "Option<DeserializeNumberLenient>")]
                    }
                    {}
                    interval, Option<i32>
                ),
                (
                    {
                        #[serde(rename = "active")]
                        #[serde_inline_default(None)]
                        #[serde_as(as = "Option<DeserializeBoolLenient>")]
                        #[derivative(PartialEq="ignore")]
                        #[derivative(Hash = "ignore")]
                    }
                    {}
                    active, Option<bool>
                ),
                (
                    {
                        #[serde(rename = "maxretries")]
                        #[serde(alias = "max_retries")]
                        #[serde_as(as = "Option<DeserializeNumberLenient>")]
                    }
                    {}
                    max_retries, Option<i32>
                ),
                (
                    {
                        #[serde(rename = "retryInterval")]
                        #[serde(alias = "retry_interval")]
                        #[serde_inline_default(Some(60))]
                        #[serde_as(as = "Option<DeserializeNumberLenient>")]
                    }
                    {}
                    retry_interval, Option<i32>
                ),
                (
                    {
                        #[serde(rename = "upsideDown")]
                        #[serde(alias = "upside_down")]
                        #[serde_as(as = "Option<DeserializeBoolLenient>")]
                    }
                    {}
                    upside_down, Option<bool>
                ),
                (
                    {
                        #[serde(rename = "parent")]
                        #[serde_as(as = "Option<DeserializeNumberLenient>")]
                        #[serialize_always]
                    }
                    {}
                    parent, Option<i32>
                ),
                (
                    {
                        #[serde(rename = "tags")]
                        #[serde(skip_serializing_if = "Vec::is_empty")]
                        #[serde(default)]
                        #[serde_as(as = "DeserializeVecLenient<Tag>")]
                        #[derivative(PartialEq(compare_with = "compare_tags"))]
                    }
                    {}
                    tags, Vec<Tag>
                ),
                (
                    {
                        #[serde(rename = "notificationIDList")]
                        #[serde(alias = "notification_id_list")]
                        #[serde_as(as = "Option<DeserializeHashMapLenient<String, bool>>")]
                    }
                    {}
                    notification_id_list, Option<HashMap<String, bool>>
                ),
                (
                    {
                        #[serde(rename = "accepted_statuscodes")]
                        #[serde_as(as = "DeserializeVecLenient<String>")]
                        #[serde_inline_default(vec!["200-299".to_owned()])]
                    }
                    {}
                    accepted_statuscodes, Vec<String>
                ),
                (
                    {
                        #[cfg(feature = "private-api")]
                        #[serde(rename = "parent_name")]
                        #[derivative(PartialEq = "ignore")]
                        #[derivative(Hash = "ignore")]
                    }
                    {
                        #[cfg(feature = "private-api")]
                    }
                    parent_name, Option<String>
                ),
                (
                    {
                        #[cfg(feature = "private-api")]
                        #[serde(rename = "create_paused")]
                        #[serde_inline_default(None)]
                        #[serde_as(as = "Option<DeserializeBoolLenient>")]
                        #[derivative(PartialEq = "ignore")]
                        #[derivative(Hash = "ignore")]
                    }
                    {
                        #[cfg(feature = "private-api")]
                    }
                    create_paused, Option<bool>
                ),
                (
                    {
                        #[cfg(feature = "private-api")]
                        #[serde(rename = "notification_name_list")]
                        #[derivative(PartialEq = "ignore")]
                        #[derivative(Hash = "ignore")]
                        #[serde_as(as = "Option<DeserializeVecLenient<String>>")]
                    }
                    {
                        #[cfg(feature = "private-api")]
                    }
                    notification_names, Option<Vec<String>>
                ),
                (
                    {
                        #[cfg(feature = "private-api")]
                        #[serde(rename = "tag_names")]
                        #[derivative(PartialEq = "ignore")]
                        #[derivative(Hash = "ignore")]
                        #[serde_as(as = "Option<DeserializeVecLenient<super::tag::TagValue>>")]
                    }
                    {
                        #[cfg(feature = "private-api")]
                    }
                    tag_names, Option<Vec<super::tag::TagValue>>
                ),
                (
                    {
                        #[cfg(not(feature = "uptime-kuma-v1"))]
                        #[serde(rename = "conditions")]
                        #[serde_as(as = "DeserializeVecLenient<MonitorCondition>")]
                        #[serde_inline_default(vec![])]
                    }
                    {
                        #[cfg(not(feature = "uptime-kuma-v1"))]
                    }
                    conditions, Vec<MonitorCondition>
                ),
                (
                    {
                        #[serde(rename = "resendInterval")]
                        #[serde(alias = "resend_interval")]
                        #[serde_as(as = "Option<DeserializeNumberLenient>")]
                    }
                    {}
                    resend_interval, Option<i32>
                )
            ]
        }
    };
}

/// Expands shared monitor fields for callback macros that produce items.
macro_rules! with_monitor_common_fields {
    ($callback:ident $($args:tt)*) => {
        with_monitor_common_fields_impl!(invoke_monitor_fields_item, $callback, $($args)*);
    };
}

/// Expands shared monitor fields for callback macros that produce expressions.
macro_rules! with_monitor_common_fields_expr {
    ($callback:ident $($args:tt)*) => {
        with_monitor_common_fields_impl!(invoke_monitor_fields_expr, $callback, $($args)*)
    };
}

/// Emits `MonitorCommon` getter and mutable getter methods from shared field metadata.
macro_rules! emit_monitor_common_methods {
    () => {};

    (({$($field_attr:tt)*} {$($cfg_attr:tt)*} $field:ident, $ty:ty) $(, $rest:tt)*) => {
        paste::paste! {
            $($cfg_attr)* fn $field(&self) -> &$ty { &self.$field }
            $($cfg_attr)* fn [<$field _mut>](&mut self) -> &mut $ty { &mut self.$field }
        }
        emit_monitor_common_methods!($($rest),*);
    };
}

/// Generates a monitor struct containing the shared monitor fields plus custom fields.
macro_rules! emit_monitor_struct {
    ($struct_name:ident $(<$($generic:ident),+>)? { $($extra:tt)* } [$(({$($field_attr:tt)*} {$($cfg_attr:tt)*} $field:ident, $ty:ty)),* $(,)?]) => {
        #[serde_inline_default]
        #[skip_serializing_none]
        #[serde_as]
        #[derive(Clone, Debug, Derivative, Serialize, Deserialize)]
        #[derivative(PartialEq)]
        pub struct $struct_name $(<$($generic),+>)? {
            $($($field_attr)* pub $field: $ty,)*
            $($extra)*
        }
    };
}

/// Implements `MonitorCommon` for a generated monitor struct.
macro_rules! emit_monitor_common_impl {
    ($struct_name:ident $(<$($generic:ident),+>)? [$(({$($field_attr:tt)*} {$($cfg_attr:tt)*} $field:ident, $ty:ty)),* $(,)?]) => {
        impl$(<$($generic),+>)? MonitorCommon for $struct_name $(<$($generic),+>)? {
            emit_monitor_common_methods!($(({$($field_attr)*} {$($cfg_attr)*} $field, $ty)),*);
        }
    };
}

/// Builds the wrapper-format value used when converting a public monitor struct.
macro_rules! emit_monitor_wrapper_value {
    ($wrapper_name:ident, $value:ident, $subtype:expr, [$(({$($field_attr:tt)*} {$($cfg_attr:tt)*} $field:ident, $ty:ty)),* $(,)?]) => {
        $wrapper_name {
            $($($cfg_attr)* $field: $value.$field,)*
            subtype: $subtype,
        }
    };
}

/// Declares a monitor struct with the shared monitor fields and custom fields.
macro_rules! define_monitor_struct {
    ($struct_name:ident $(<$($generic:ident),+>)? {
        $($field:tt)*
    }) => {
        with_monitor_common_fields!(emit_monitor_struct $struct_name $(<$($generic),+>)? { $($field)* });
    };
}

/// Declares a monitor struct and, for generic wrappers, its conversion into a wrapper struct.
macro_rules! monitor_struct {
    ($struct_name:ident <$($generic:ident),+> => $wrapper_name:ident where ($($bounds:tt)+) {
        $($field:tt)*
    }) => {
        define_monitor_struct!($struct_name<$($generic),+> { $($field)* });

        impl<$($generic),+> From<$struct_name<$($generic),+>> for $wrapper_name
        where
            $($bounds)+
        {
            fn from(value: $struct_name<$($generic),+>) -> Self {
                let wrapper = with_monitor_common_fields_expr!(
                    emit_monitor_wrapper_value $wrapper_name, value, value.subtype.into(),
                );
                wrapper
            }
        }
    };

    ($struct_name:ident $(<$($generic:ident),+>)? {
        $($field:tt)*
    }) => {
        define_monitor_struct!($struct_name $(<$($generic),+>)? { $($field)* });
    };
}

/// Implements `MonitorCommon` for a previously declared monitor struct.
macro_rules! impl_monitor_common {
    ($struct_name:ident $(<$($generic:ident),+>)?) => {
        with_monitor_common_fields!(emit_monitor_common_impl $struct_name $(<$($generic),+>)?);
    };
}

/// Declares a concrete monitor type, its `MonitorCommon` impl, and conversion into `Monitor`.
macro_rules! monitor_type {
    ($struct_name:ident $type:ident {
        $($field:tt)*
    }) => {
        define_monitor_struct!($struct_name { $($field)* });

        impl_monitor_common!($struct_name);

        impl From<$struct_name> for Monitor {
            fn from(value: $struct_name) -> Self {
                Monitor::$type { value: value }
            }
        }

        crate::default_from_serde!($struct_name);
    };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MonitorType {
    #[serde(rename = "dns")]
    Dns,

    #[serde(rename = "docker")]
    Docker,

    #[serde(rename = "gamedig")]
    GameDig,

    #[serde(rename = "globalping")]
    GlobalPing,

    #[serde(rename = "group")]
    Group,

    #[serde(rename = "grpc-keyword")]
    GrpcKeyword,

    #[serde(rename = "http")]
    Http,

    #[serde(rename = "json-query")]
    JsonQuery,

    #[serde(rename = "kafka-producer")]
    KafkaProducer,

    #[serde(rename = "keyword")]
    Keyword,

    #[serde(rename = "mongodb")]
    Mongodb,

    #[serde(rename = "mqtt")]
    Mqtt,

    #[serde(rename = "mysql")]
    Mysql,

    #[serde(rename = "ping")]
    Ping,

    #[serde(rename = "port")]
    Port,

    #[serde(rename = "postgres")]
    Postgres,

    #[serde(rename = "push")]
    Push,

    #[serde(rename = "radius")]
    Radius,

    #[serde(rename = "real-browser")]
    RealBrowser,

    #[serde(rename = "redis")]
    Redis,

    #[serde(rename = "steam")]
    Steam,

    #[serde(rename = "sqlserver")]
    SqlServer,

    #[serde(rename = "tailscale-ping")]
    TailscalePing,

    #[cfg(not(feature = "uptime-kuma-v1"))]
    #[serde(rename = "smtp")]
    SMTP,

    #[cfg(not(feature = "uptime-kuma-v1"))]
    #[serde(rename = "snmp")]
    SNMP,

    #[cfg(not(feature = "uptime-kuma-v1"))]
    #[serde(rename = "rabbitmq")]
    RabbitMQ,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DnsResolverType {
    A,
    AAAA,
    CAA,
    CNAME,
    MX,
    NS,
    PTR,
    SOA,
    SRV,
    TXT,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GlobalPingIpFamily {
    #[serde(rename = "ipv4")]
    IPv4,

    #[serde(rename = "ipv6")]
    IPv6,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    DELETE,
    GET,
    HEAD,
    OPTIONS,
    PATCH,
    POST,
    PUT,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mechanism")]
pub enum KafkaProducerSaslOptions {
    #[serde(rename = "None")]
    None,

    #[serde(rename = "plain")]
    Plain {
        #[serde(rename = "username")]
        username: Option<String>,

        #[serde(rename = "password")]
        password: Option<String>,
    },

    #[serde(rename = "scram-sha-256")]
    ScramSha256 {
        #[serde(rename = "username")]
        username: Option<String>,

        #[serde(rename = "password")]
        password: Option<String>,
    },

    #[serde(rename = "scram-sha-512")]
    ScramSha512 {
        #[serde(rename = "username")]
        username: Option<String>,

        #[serde(rename = "password")]
        password: Option<String>,
    },

    #[serde(rename = "aws")]
    AWS {
        #[serde(rename = "authorizationIdentity")]
        #[serde(alias = "authorization_identity")]
        authorization_identity: Option<String>,

        #[serde(rename = "accessKeyId")]
        #[serde(alias = "access_key_id")]
        access_key_id: Option<String>,

        #[serde(rename = "secretAccessKey")]
        #[serde(alias = "secret_access_key")]
        secret_access_key: Option<String>,

        #[serde(alias = "sessionToken")]
        #[serde(rename = "session_token")]
        session_token: Option<String>,
    },
}

fn compare_tags(a: &Vec<Tag>, b: &Vec<Tag>) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.iter().collect::<HashSet<_>>() == b.iter().collect::<HashSet<_>>()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HttpOAuthMethod {
    #[serde(rename = "client_secret_basic")]
    ClientSecretBasic,

    #[serde(rename = "client_secret_post")]
    ClientSecretPost,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "authMethod")]
pub enum HttpAuth {
    #[serde(rename = "null")]
    None,

    #[serde(rename = "basic")]
    Basic {
        #[serde(rename = "basic_auth_user")]
        username: Option<String>,

        #[serde(rename = "basic_auth_pass")]
        password: Option<String>,
    },

    #[serde(rename = "oauth2-cc")]
    OAuth2 {
        #[serde(rename = "oauth_auth_method")]
        method: Option<HttpOAuthMethod>,

        #[serde(rename = "oauth_client_id")]
        client_id: Option<String>,

        #[serde(rename = "oauth_token_url")]
        token_url: Option<String>,

        #[serde(rename = "oauth_client_secret")]
        client_secret: Option<String>,

        #[serde(rename = "oauth_scopes")]
        scopes: Option<String>,

        #[serde(rename = "oauth_audience")]
        audience: Option<String>,
    },

    #[serde(rename = "ntlm")]
    NTLM {
        #[serde(rename = "basic_auth_user")]
        basic_auth_user: Option<String>,

        #[serde(rename = "basic_auth_pass")]
        basic_auth_pass: Option<String>,

        #[serde(rename = "authDomain")]
        #[serde(alias = "auth_domain")]
        auth_domain: Option<String>,

        #[serde(rename = "authWorkstation")]
        #[serde(alias = "auth_workstation")]
        auth_workstation: Option<String>,
    },

    #[serde(rename = "mtls")]
    MTLS {
        #[serde(rename = "tlsCert")]
        #[serde(alias = "tls_cert")]
        tls_cert: Option<String>,

        #[serde(rename = "tlsKey")]
        #[serde(alias = "tls_key")]
        tls_key: Option<String>,

        #[serde(rename = "tlsCa")]
        #[serde(alias = "tls_ca")]
        tls_ca: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MonitorConditionOperator {
    #[serde(rename = "equals")]
    Equals,
    #[serde(rename = "not_equals")]
    NotEquals,
    #[serde(rename = "contains")]
    Contains,
    #[serde(rename = "not_contains")]
    NotContains,
    #[serde(rename = "starts_with")]
    StartsWith,
    #[serde(rename = "not_starts_with")]
    NotStartsWith,
    #[serde(rename = "ends_with")]
    EndsWith,
    #[serde(rename = "not_ends_with")]
    NotEndsWith,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MonitorConditionConjunction {
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitorCondition {
    #[serde(rename = "expression")]
    Expression {
        #[serde(rename = "variable")]
        variable: Option<String>,
        #[serde(rename = "operator")]
        operator: Option<MonitorConditionOperator>,
        #[serde(rename = "value")]
        value: Option<String>,
        #[serde(rename = "andOr")]
        conjunction: Option<MonitorConditionConjunction>,
    },

    #[serde(rename = "group")]
    Group {
        #[serde(rename = "children")]
        children: Option<Vec<MonitorCondition>>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SMTPSecurity {
    #[serde(rename = "secure")]
    SMTPS,

    #[serde(rename = "nostarttls")]
    NOTLS,

    #[serde(rename = "starttls")]
    STARTTLS,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SNMPVersion {
    #[serde(rename = "1")]
    SNMPv1,

    #[serde(rename = "2c")]
    SNMPv2c,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum HttpBodyEncoding {
    #[default]
    #[serde(rename = "json")]
    Json,

    #[cfg(not(feature = "uptime-kuma-v1"))]
    #[serde(rename = "form")]
    Form,

    #[serde(rename = "xml")]
    Xml,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum MqttCheckType {
    #[default]
    #[serde(rename = "keyword")]
    Keyword,

    #[serde(rename = "json-query")]
    JsonQuery,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum JsonPathOperator {
    #[serde(rename = ">")]
    Greater,

    #[serde(rename = ">=")]
    GreaterEqual,

    #[serde(rename = "<")]
    Less,

    #[serde(rename = "<=")]
    LessEqual,

    #[serde(rename = "!=")]
    NotEqual,

    #[default]
    #[serde(rename = "==")]
    Equal,

    #[serde(rename = "contains")]
    Contains,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ExpectedValue {
    String(String),
    Number(f64),
}

impl Serialize for ExpectedValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ExpectedValue::String(ref s) => serializer.serialize_str(s),
            ExpectedValue::Number(n) => serializer.serialize_str(&n.to_string()),
        }
    }
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalPingPing {
    #[serde(rename = "location")]
    #[serde_inline_default(Some("world".to_owned()))]
    pub location: Option<String>,

    #[serde(rename = "hostname")]
    pub hostname: Option<String>,

    #[serde(rename = "protocol")]
    pub protocol: Option<String>,

    #[serde(rename = "ipFamily")]
    #[serde(alias = "ip_family")]
    pub ip_family: Option<GlobalPingIpFamily>,

    #[serde(rename = "port")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub port: Option<u16>,

    #[serde(rename = "ping_count")]
    #[serde_inline_default(Some(3))]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub ping_count: Option<i32>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalPingHttp {
    #[serde(rename = "location")]
    #[serde_inline_default(Some("world".to_owned()))]
    pub location: Option<String>,

    #[serde(rename = "url")]
    pub url: Option<String>,

    #[serde(rename = "method")]
    #[serde_inline_default(Some(HttpMethod::GET))]
    pub method: Option<HttpMethod>,

    #[serde(rename = "protocol")]
    pub protocol: Option<String>,

    #[serde(rename = "ipFamily")]
    #[serde(alias = "ip_family")]
    pub ip_family: Option<GlobalPingIpFamily>,

    #[serde(rename = "dns_resolve_server")]
    pub dns_resolve_server: Option<String>,

    #[serde(rename = "headers")]
    pub headers: Option<String>,

    #[serde(flatten)]
    pub auth: Option<HttpAuth>,

    #[serde(rename = "expiryNotification")]
    #[serde(alias = "expiry_notification")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub expiry_notification: Option<bool>,

    #[serde(rename = "ignoreTls")]
    #[serde(alias = "ignore_tls")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub ignore_tls: Option<bool>,

    #[serde(rename = "cacheBust")]
    #[serde(alias = "cache_bust")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub cache_bust: Option<bool>,

    #[serde(rename = "keyword")]
    pub keyword: Option<String>,

    #[serde(rename = "invertKeyword")]
    #[serde(alias = "invert_keyword")]
    #[serde_as(as = "Option<DeserializeBoolLenient>")]
    pub invert_keyword: Option<bool>,

    #[serde(rename = "jsonPath")]
    #[serde(alias = "json_path")]
    pub json_path: Option<String>,

    #[serde(rename = "jsonPathOperator")]
    #[serde(alias = "json_path_operator")]
    pub json_path_operator: Option<JsonPathOperator>,

    #[serde(rename = "expectedValue")]
    #[serde(alias = "expected_value")]
    pub expected_value: Option<ExpectedValue>,
}

#[serde_inline_default]
#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalPingDns {
    #[serde(rename = "location")]
    #[serde_inline_default(Some("world".to_owned()))]
    pub location: Option<String>,

    #[serde(rename = "hostname")]
    pub hostname: Option<String>,

    #[serde(rename = "protocol")]
    pub protocol: Option<String>,

    #[serde(rename = "ipFamily")]
    #[serde(alias = "ip_family")]
    pub ip_family: Option<GlobalPingIpFamily>,

    #[serde(rename = "dns_resolve_server")]
    pub dns_resolve_server: Option<String>,

    #[serde(rename = "port")]
    #[serde_as(as = "Option<DeserializeNumberLenient>")]
    pub port: Option<u16>,

    #[serde(rename = "dns_resolve_type")]
    #[serde_inline_default(Some("A".to_owned()))]
    pub dns_resolve_type: Option<String>,

    #[serde(rename = "dns_last_result")]
    pub dns_last_result: Option<String>,

    #[serde(rename = "keyword")]
    pub keyword: Option<String>,
}

crate::default_from_serde!(GlobalPingPing);
crate::default_from_serde!(GlobalPingHttp);
crate::default_from_serde!(GlobalPingDns);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::From)]
#[serde(tag = "subtype")]
pub enum GlobalPingSubtype {
    #[serde(rename = "ping")]
    Ping {
        #[serde(flatten)]
        value: GlobalPingPing,
    },

    #[serde(rename = "http")]
    Http {
        #[serde(flatten)]
        value: GlobalPingHttp,
    },

    #[serde(rename = "dns")]
    Dns {
        #[serde(flatten)]
        value: GlobalPingDns,
    },
}

impl Default for GlobalPingSubtype {
    fn default() -> Self {
        Self::Ping {
            value: GlobalPingPing::default(),
        }
    }
}

monitor_type! {
    MonitorGroup Group {

    }
}

monitor_type! {
    MonitorSqlServer SqlServer {
        #[serde(rename = "databaseConnectionString")]
        #[serde(alias = "database_connection_string")]
        pub database_connection_string: Option<String>,

        #[serde(rename = "databaseQuery")]
        #[serde(alias = "query")]
        pub query: Option<String>,
    }
}

monitor_type! {
    MonitorPostgres Postgres {
        #[serde(rename = "databaseConnectionString")]
        #[serde(alias = "database_connection_string")]
        pub database_connection_string: Option<String>,

        #[serde(rename = "databaseQuery")]
        #[serde(alias = "query")]
        pub query: Option<String>,
    }
}

monitor_type! {
    MonitorMongoDB Mongodb {
        #[serde(rename = "databaseConnectionString")]
        #[serde(alias = "database_connection_string")]
        pub database_connection_string: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "databaseQuery")]
        #[serde(alias = "command")]
        pub command: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "jsonPath")]
        #[serde(alias = "json_path")]
        pub json_path: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "expectedValue")]
        #[serde(alias = "expected_value")]
        pub expected_value: Option<ExpectedValue>,
    }
}

monitor_type! {
    MonitorMysql Mysql {
        #[serde(rename = "databaseConnectionString")]
        #[serde(alias = "database_connection_string")]
        pub database_connection_string: Option<String>,

        #[serde(rename = "radiusPassword")]
        #[serde(alias = "radius_password")]
        pub password: Option<String>,

        #[serde(rename = "databaseQuery")]
        #[serde(alias = "query")]
        pub query: Option<String>,
    }
}

monitor_type! {
    MonitorRedis Redis {
        #[serde(rename = "databaseConnectionString")]
        #[serde(alias = "database_connection_string")]
        pub database_connection_string: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "ignoreTls")]
        #[serde(alias = "ignore_tls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub ignore_tls: Option<bool>,
    }
}

monitor_type! {
    MonitorDns Dns {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "dns_resolve_server")]
        #[serde_inline_default(Some("1.1.1.1".to_owned()))]
        pub dns_resolve_server: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,

        #[serde(rename = "dns_resolve_type")]
        #[serde_inline_default(Some(DnsResolverType::A))]
        pub dns_resolve_type: Option<DnsResolverType>,
    }
}

monitor_type! {
    MonitorDocker Docker {
        #[serde(rename = "docker_container")]
        pub docker_container: Option<String>,

        #[serde(rename = "docker_host")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub docker_host: Option<i32>,

        #[cfg(feature = "private-api")]
        #[serde(rename = "docker_host_name")]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        pub docker_host_name: Option<String>,
    }
}

monitor_type! {
    MonitorGameDig GameDig {
        #[serde(rename = "game")]
        pub game: Option<String>,

        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,

        #[serde(rename = "gamedigGivenPortOnly")]
        #[serde(alias = "gamedig_given_port_only")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub gamedig_given_port_only: Option<bool>,
    }
}

monitor_type! {
    MonitorGlobalPingWrapper GlobalPing {
        #[serde(flatten)]
        #[serde(default)]
        pub subtype: GlobalPingSubtype,
    }
}

monitor_struct!(MonitorGlobalPing<T> => MonitorGlobalPingWrapper where (T: Into<GlobalPingSubtype>) {
    #[serde(flatten)]
    #[serde(default)]
    pub subtype: T,
});

impl_monitor_common!(MonitorGlobalPing<T>);

crate::default_from_serde!(MonitorGlobalPing<T> where T: Default + serde::de::DeserializeOwned);

impl<T> From<MonitorGlobalPing<T>> for Monitor
where
    T: Into<GlobalPingSubtype>,
{
    fn from(value: MonitorGlobalPing<T>) -> Self {
        Monitor::GlobalPing { value: value.into() }
    }
}

monitor_type! {
    MonitorGrpcKeyword GrpcKeyword {
        #[serde(rename = "keyword")]
        pub keyword: Option<String>,

        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub invert_keyword: Option<bool>,

        #[serde(rename = "grpcUrl")]
        #[serde(alias = "grpc_url")]
        pub grpc_url: Option<String>,

        #[serde(rename = "maxredirects")]
        #[serde(alias = "max_redirects")]
        #[serde_inline_default(Some(10))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "grpcEnableTls")]
        #[serde(alias = "grpc_enable_tls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub grpc_enable_tls: Option<bool>,

        #[serde(rename = "grpcServiceName")]
        #[serde(alias = "grpc_service_name")]
        pub grpc_service_name: Option<String>,

        #[serde(rename = "grpcMethod")]
        #[serde(alias = "grpc_method")]
        pub grpc_method: Option<String>,

        #[serde(rename = "grpcProtobuf")]
        #[serde(alias = "grpc_protobuf")]
        pub grpc_protobuf: Option<String>,

        #[serde(rename = "grpcBody")]
        #[serde(alias = "grpc_body")]
        pub grpc_body: Option<String>,

        #[serde(rename = "grpcMetadata")]
        #[serde(alias = "grpc_metadata")]
        pub grpc_metadata: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "cacheBust")]
        #[serde(alias = "cache_bust")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub cache_bust: Option<bool>,
    }
}

monitor_type! {
    MonitorHttp Http {
        #[serde(rename = "url")]
        pub url: Option<String>,

        #[serde(rename = "timeout")]
        #[serde_inline_default(Some(48))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub timeout: Option<i32>,

        #[serde(rename = "expiryNotification")]
        #[serde(alias = "expiry_notification")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub expiry_notification: Option<bool>,

        #[serde(rename = "ignoreTls")]
        #[serde(alias = "ignore_tls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub ignore_tls: Option<bool>,

        #[serde(rename = "maxredirects")]
        #[serde(alias = "max_redirects")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "proxyId")]
        #[serde(alias = "proxy_id")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub proxy_id: Option<i32>,

        #[serde(rename = "method")]
        #[serde_inline_default(Some(HttpMethod::GET))]
        pub method: Option<HttpMethod>,

        #[serde(rename = "httpBodyEncoding")]
        #[serde(alias = "http_body_encoding")]
        pub http_body_encoding: Option<HttpBodyEncoding>,

        #[serde(rename = "body")]
        pub body: Option<String>,

        #[serde(rename = "headers")]
        pub headers: Option<String>,

        #[serde(flatten)]
        pub auth: Option<HttpAuth>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "cacheBust")]
        #[serde(alias = "cache_bust")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub cache_bust: Option<bool>,
    }
}

monitor_type! {
    MonitorJsonQuery JsonQuery {
        #[serde(rename = "jsonPath")]
        #[serde(alias = "json_path")]
        pub json_path: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "jsonPathOperator")]
        #[serde(alias = "json_path_operator")]
        pub json_path_operator: Option<JsonPathOperator>,

        #[serde(rename = "expectedValue")]
        #[serde(alias = "expected_value")]
        pub expected_value: Option<ExpectedValue>,

        #[serde(rename = "url")]
        pub url: Option<String>,

        #[serde(rename = "timeout")]
        #[serde_inline_default(Some(48))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub timeout: Option<i32>,

        #[serde(rename = "expiryNotification")]
        #[serde(alias = "expiry_notification")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub expiry_notification: Option<bool>,

        #[serde(rename = "ignoreTls")]
        #[serde(alias = "ignore_tls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub ignore_tls: Option<bool>,

        #[serde(rename = "maxredirects")]
        #[serde(alias = "max_redirects")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "proxyId")]
        #[serde(alias = "proxy_id")]
        pub proxy_id: Option<String>,

        #[serde(rename = "method")]
        #[serde_inline_default(Some(HttpMethod::GET))]
        pub method: Option<HttpMethod>,

        #[serde(rename = "httpBodyEncoding")]
        #[serde(alias = "http_body_encoding")]
        pub http_body_encoding: Option<HttpBodyEncoding>,

        #[serde(rename = "body")]
        pub body: Option<String>,

        #[serde(rename = "headers")]
        pub headers: Option<String>,

        #[serde(flatten)]
        pub auth: Option<HttpAuth>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "grpcMetadata")]
        #[serde(alias = "grpc_metadata")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub cache_bust: Option<bool>,
    }
}

monitor_type! {
    MonitorKafkaProducer KafkaProducer {
        #[serde(rename = "kafkaProducerBrokers")]
        #[serde(alias = "kafka_producer_brokers")]
        #[serde_as(as = "DeserializeVecLenient<String>")]
        pub kafka_producer_brokers: Vec<String>,

        #[serde(rename = "kafkaProducerTopic")]
        #[serde(alias = "kafka_producer_topic")]
        pub kafka_producer_topic: Option<String>,

        #[serde(rename = "kafkaProducerMessage")]
        #[serde(alias = "kafka_producer_message")]
        pub kafka_producer_message: Option<String>,

        #[serde(rename = "kafkaProducerSsl")]
        #[serde(alias = "kafka_producer_ssl")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub kafka_producer_ssl: Option<bool>,

        #[serde(rename = "kafkaProducerAllowAutoTopicCreation")]
        #[serde(alias = "kafka_producer_allow_auto_topic_creation")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub kafka_producer_allow_auto_topic_creation: Option<bool>,

        #[serde(rename = "kafkaProducerSaslOptions")]
        #[serde(alias = "kafka_producer_sasl_options")]
        pub kafka_producer_sasl_options: Option<KafkaProducerSaslOptions>,
    }
}

monitor_type! {
    MonitorKeyword Keyword {
        #[serde(rename = "keyword")]
        pub keyword: Option<String>,

        #[serde(rename = "invertKeyword")]
        #[serde(alias = "invert_keyword")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub invert_keyword: Option<bool>,

        #[serde(rename = "url")]
        pub url: Option<String>,

        #[serde(rename = "timeout")]
        #[serde_inline_default(Some(48))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub timeout: Option<i32>,

        #[serde(rename = "expiryNotification")]
        #[serde(alias = "expiry_notification")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub expiry_notification: Option<bool>,

        #[serde(rename = "ignoreTls")]
        #[serde(alias = "ignore_tls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub ignore_tls: Option<bool>,

        #[serde(rename = "maxredirects")]
        #[serde(alias = "max_redirects")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "proxyId")]
        #[serde(alias = "proxy_id")]
        pub proxy_id: Option<String>,

        #[serde(rename = "method")]
        #[serde_inline_default(Some(HttpMethod::GET))]
        pub method: Option<HttpMethod>,

        #[serde(rename = "httpBodyEncoding")]
        #[serde(alias = "http_body_encoding")]
        pub http_body_encoding: Option<HttpBodyEncoding>,

        #[serde(rename = "body")]
        pub body: Option<String>,

        #[serde(rename = "headers")]
        pub headers: Option<String>,

        #[serde(flatten)]
        pub auth: Option<HttpAuth>,
    }
}

monitor_type! {
    MonitorMqtt Mqtt {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,

        #[serde(rename = "mqttUsername")]
        #[serde(alias = "mqtt_username")]
        pub mqtt_username: Option<String>,

        #[serde(rename = "mqttPassword")]
        #[serde(alias = "mqtt_password")]
        pub mqtt_password: Option<String>,

        #[serde(rename = "mqttTopic")]
        #[serde(alias = "mqtt_topic")]
        pub mqtt_topic: Option<String>,

        #[serde(rename = "mqttCheckType")]
        #[serde(alias = "mqtt_check_type")]
        pub mqtt_check_type: Option<MqttCheckType>,

        #[serde(rename = "mqttSuccessMessage")]
        #[serde(alias = "mqtt_success_message")]
        pub mqtt_success_message: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "jsonPath")]
        #[serde(alias = "json_path")]
        pub json_path: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "jsonPathOperator")]
        #[serde(alias = "json_path_operator")]
        pub json_path_operator: Option<JsonPathOperator>,

        #[serde(rename = "expectedValue")]
        #[serde(alias = "expected_value")]
        pub expected_value: Option<ExpectedValue>,
    }
}

monitor_type! {
    MonitorPing Ping {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "packetSize")]
        #[serde(alias = "packet_size")]
        #[serde_inline_default(Some(56))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub packet_size: Option<i32>,
    }
}

monitor_type! {
    MonitorPort Port {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,
    }
}

monitor_type! {
    MonitorPush Push {
        #[serde(rename = "pushToken")]
        #[serde(alias = "push_token")]
        pub push_token: Option<String>,
    }
}

monitor_type! {
    MonitorRadius Radius {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,

        #[serde(rename = "radiusUsername")]
        #[serde(alias = "radius_username")]
        pub radius_username: Option<String>,

        #[serde(rename = "radiusPassword")]
        #[serde(alias = "radius_password")]
        pub radius_password: Option<String>,

        #[serde(rename = "radiusSecret")]
        #[serde(alias = "radius_secret")]
        pub radius_secret: Option<String>,

        #[serde(rename = "radiusCalledStationId")]
        #[serde(alias = "radius_called_station_id")]
        pub radius_called_station_id: Option<String>,

        #[serde(rename = "radiusCallingStationId")]
        #[serde(alias = "radius_calling_station_id")]
        pub radius_calling_station_id: Option<String>,
    }
}

monitor_type! {
    MonitorRealBrowser RealBrowser {
        #[serde(rename = "url")]
        pub url: Option<String>,

        #[serde(rename = "remoteBrowsersToggle")]
        #[serde(alias = "remote_browsers_toggle")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub remote_browsers_toggle: Option<bool>,

        #[serde(rename = "remote_browser")]
        pub remote_browser: Option<String>,
    }
}

monitor_type! {
    MonitorSteam Steam {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,
    }
}

monitor_type! {
    MonitorTailscalePing TailscalePing {
        #[serde(rename = "hostname")]
        hostname: Option<String>,
    }
}

#[cfg(not(feature = "uptime-kuma-v1"))]
monitor_type! {
    MonitorSMTP SMTP {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,

        #[serde(rename = "smtp_security")]
        #[serde(alias = "security")]
        pub security: Option<SMTPSecurity>,
    }
}

#[cfg(not(feature = "uptime-kuma-v1"))]
monitor_type! {
    MonitorSNMP SNMP {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub port: Option<u16>,

        #[serde(rename = "radiusPassword")]
        #[serde(alias = "radius_password")]
        pub password: Option<String>,

        #[serde(rename = "snmpOid")]
        #[serde(alias = "oid")]
        pub oid: Option<String>,

        #[serde(rename = "snmp_version")]
        #[serde(alias = "version")]
        pub version: Option<SNMPVersion>,

        #[serde(rename = "jsonPath")]
        #[serde(alias = "json_path")]
        pub json_path: Option<String>,

        #[cfg(not(feature = "uptime-kuma-v1"))]
        #[serde(rename = "jsonPathOperator")]
        #[serde(alias = "json_path_operator")]
        pub json_path_operator: Option<JsonPathOperator>,

        #[serde(rename = "expectedValue")]
        #[serde(alias = "expected_value")]
        pub expected_value: Option<ExpectedValue>,
    }
}

#[cfg(not(feature = "uptime-kuma-v1"))]
monitor_type! {
    MonitorRabbitMQ RabbitMQ {
        #[serde(rename = "rabbitmqNodes")]
        #[serde_as(as = "DeserializeVecLenient<String>")]
        #[serde_inline_default(vec![])]
        pub nodes: Vec<String>,

        #[serde(rename = "rabbitmqUsername")]
        #[serde(alias = "username")]
        pub username: Option<String>,

        #[serde(rename = "rabbitmqPassword")]
        #[serde(alias = "password")]
        pub password: Option<String>,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Monitor {
    #[serde(rename = "group")]
    Group {
        #[serde(flatten)]
        value: MonitorGroup,
    },

    #[serde(rename = "http")]
    Http {
        #[serde(flatten)]
        value: MonitorHttp,
    },

    #[serde(rename = "port")]
    Port {
        #[serde(flatten)]
        value: MonitorPort,
    },

    #[serde(rename = "ping")]
    Ping {
        #[serde(flatten)]
        value: MonitorPing,
    },

    #[serde(rename = "keyword")]
    Keyword {
        #[serde(flatten)]
        value: MonitorKeyword,
    },

    #[serde(rename = "json-query")]
    JsonQuery {
        #[serde(flatten)]
        value: MonitorJsonQuery,
    },

    #[serde(rename = "grpc-keyword")]
    GrpcKeyword {
        #[serde(flatten)]
        value: MonitorGrpcKeyword,
    },

    #[serde(rename = "dns")]
    Dns {
        #[serde(flatten)]
        value: MonitorDns,
    },

    #[serde(rename = "docker")]
    Docker {
        #[serde(flatten)]
        value: MonitorDocker,
    },

    #[serde(rename = "real-browser")]
    RealBrowser {
        #[serde(flatten)]
        value: MonitorRealBrowser,
    },

    #[serde(rename = "push")]
    Push {
        #[serde(flatten)]
        value: MonitorPush,
    },

    #[serde(rename = "steam")]
    Steam {
        #[serde(flatten)]
        value: MonitorSteam,
    },

    #[serde(rename = "gamedig")]
    GameDig {
        #[serde(flatten)]
        value: MonitorGameDig,
    },

    #[serde(rename = "globalping")]
    GlobalPing {
        #[serde(flatten)]
        value: MonitorGlobalPingWrapper,
    },

    #[serde(rename = "mqtt")]
    Mqtt {
        #[serde(flatten)]
        value: MonitorMqtt,
    },

    #[serde(rename = "kafka-producer")]
    KafkaProducer {
        #[serde(flatten)]
        value: MonitorKafkaProducer,
    },

    #[serde(rename = "sqlserver")]
    SqlServer {
        #[serde(flatten)]
        value: MonitorSqlServer,
    },

    #[serde(rename = "postgres")]
    Postgres {
        #[serde(flatten)]
        value: MonitorPostgres,
    },

    #[serde(rename = "mysql")]
    Mysql {
        #[serde(flatten)]
        value: MonitorMysql,
    },

    #[serde(rename = "mongodb")]
    Mongodb {
        #[serde(flatten)]
        value: MonitorMongoDB,
    },

    #[serde(rename = "radius")]
    Radius {
        #[serde(flatten)]
        value: MonitorRadius,
    },

    #[serde(rename = "redis")]
    Redis {
        #[serde(flatten)]
        value: MonitorRedis,
    },

    #[serde(rename = "tailscale-ping")]
    TailscalePing {
        #[serde(flatten)]
        value: MonitorTailscalePing,
    },

    #[cfg(not(feature = "uptime-kuma-v1"))]
    #[serde(rename = "smtp")]
    SMTP {
        #[serde(flatten)]
        value: MonitorSMTP,
    },

    #[cfg(not(feature = "uptime-kuma-v1"))]
    #[serde(rename = "snmp")]
    SNMP {
        #[serde(flatten)]
        value: MonitorSNMP,
    },
    #[cfg(not(feature = "uptime-kuma-v1"))]
    #[serde(rename = "rabbitmq")]
    RabbitMQ {
        #[serde(flatten)]
        value: MonitorRabbitMQ,
    },
}

impl Monitor {
    pub fn monitor_type(&self) -> MonitorType {
        match self {
            Monitor::Group { .. } => MonitorType::Group,
            Monitor::Http { .. } => MonitorType::Http,
            Monitor::Port { .. } => MonitorType::Port,
            Monitor::Ping { .. } => MonitorType::Ping,
            Monitor::Keyword { .. } => MonitorType::Keyword,
            Monitor::JsonQuery { .. } => MonitorType::JsonQuery,
            Monitor::GrpcKeyword { .. } => MonitorType::GrpcKeyword,
            Monitor::Dns { .. } => MonitorType::Dns,
            Monitor::Docker { .. } => MonitorType::Docker,
            Monitor::RealBrowser { .. } => MonitorType::RealBrowser,
            Monitor::Push { .. } => MonitorType::Push,
            Monitor::Steam { .. } => MonitorType::Steam,
            Monitor::GameDig { .. } => MonitorType::GameDig,
            Monitor::GlobalPing { .. } => MonitorType::GlobalPing,
            Monitor::Mqtt { .. } => MonitorType::Mqtt,
            Monitor::KafkaProducer { .. } => MonitorType::KafkaProducer,
            Monitor::SqlServer { .. } => MonitorType::SqlServer,
            Monitor::Postgres { .. } => MonitorType::Postgres,
            Monitor::Mysql { .. } => MonitorType::Mysql,
            Monitor::Mongodb { .. } => MonitorType::Mongodb,
            Monitor::Radius { .. } => MonitorType::Radius,
            Monitor::Redis { .. } => MonitorType::Redis,
            Monitor::TailscalePing { .. } => MonitorType::TailscalePing,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::SMTP { .. } => MonitorType::SMTP,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::SNMP { .. } => MonitorType::SNMP,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::RabbitMQ { .. } => MonitorType::RabbitMQ,
        }
    }

    pub fn common(&self) -> &dyn MonitorCommon {
        match self {
            Monitor::Group { value } => value,
            Monitor::Http { value } => value,
            Monitor::Port { value } => value,
            Monitor::Ping { value } => value,
            Monitor::Keyword { value } => value,
            Monitor::JsonQuery { value } => value,
            Monitor::GrpcKeyword { value } => value,
            Monitor::Dns { value } => value,
            Monitor::Docker { value } => value,
            Monitor::RealBrowser { value } => value,
            Monitor::Push { value } => value,
            Monitor::Steam { value } => value,
            Monitor::GameDig { value } => value,
            Monitor::GlobalPing { value } => value,
            Monitor::Mqtt { value } => value,
            Monitor::KafkaProducer { value } => value,
            Monitor::SqlServer { value } => value,
            Monitor::Postgres { value } => value,
            Monitor::Mysql { value } => value,
            Monitor::Mongodb { value } => value,
            Monitor::Radius { value } => value,
            Monitor::Redis { value } => value,
            Monitor::TailscalePing { value } => value,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::SMTP { value } => value,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::SNMP { value } => value,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::RabbitMQ { value } => value,
        }
    }

    pub fn common_mut(&mut self) -> &mut dyn MonitorCommon {
        match self {
            Monitor::Group { value } => value,
            Monitor::Http { value } => value,
            Monitor::Port { value } => value,
            Monitor::Ping { value } => value,
            Monitor::Keyword { value } => value,
            Monitor::JsonQuery { value } => value,
            Monitor::GrpcKeyword { value } => value,
            Monitor::Dns { value } => value,
            Monitor::Docker { value } => value,
            Monitor::RealBrowser { value } => value,
            Monitor::Push { value } => value,
            Monitor::Steam { value } => value,
            Monitor::GameDig { value } => value,
            Monitor::GlobalPing { value } => value,
            Monitor::Mqtt { value } => value,
            Monitor::KafkaProducer { value } => value,
            Monitor::SqlServer { value } => value,
            Monitor::Postgres { value } => value,
            Monitor::Mysql { value } => value,
            Monitor::Mongodb { value } => value,
            Monitor::Radius { value } => value,
            Monitor::Redis { value } => value,
            Monitor::TailscalePing { value } => value,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::SMTP { value } => value,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::SNMP { value } => value,
            #[cfg(not(feature = "uptime-kuma-v1"))]
            Monitor::RabbitMQ { value } => value,
        }
    }

    pub fn validate(&self, id: impl AsRef<str>) -> Result<()> {
        let mut errors = vec![];

        if self.common().name().is_none() {
            errors.push("Missing property 'name'".to_owned());
        }

        if let &Monitor::Push { value } = &self {
            if let Some(push_token) = &value.push_token {
                let regex = Regex::new("^[A-Za-z0-9]{32}$").unwrap();
                if !regex.is_match(&push_token) {
                    errors.push("Invalid push_token, push token should be 32 characters and contain only letters and numbers".to_owned());
                }
            }
        }

        if !errors.is_empty() {
            return Err(Error::ValidationError(id.as_ref().to_owned(), errors));
        }

        Ok(())
    }
}

pub type MonitorList = HashMap<String, Monitor>;
