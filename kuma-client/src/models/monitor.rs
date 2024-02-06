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
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use serde_json::json;
use serde_with::{serde_as, skip_serializing_none};
use std::collections::{HashMap, HashSet};

pub trait MonitorCommon {
    fn id(&self) -> &Option<i32>;
    fn id_mut(&mut self) -> &mut Option<i32>;
    fn name(&self) -> &Option<String>;
    fn name_mut(&mut self) -> &mut Option<String>;
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
    fn parent_name(&self) -> &Option<String>;
    fn parent_name_mut(&mut self) -> &mut Option<String>;
    fn tags(&self) -> &Vec<Tag>;
    fn tags_mut(&mut self) -> &mut Vec<Tag>;
    fn notification_id_list(&self) -> &Option<HashMap<String, bool>>;
    fn notification_id_list_mut(&mut self) -> &mut Option<HashMap<String, bool>>;
    fn accepted_statuscodes(&self) -> &Vec<String>;
    fn accepted_statuscodes_mut(&mut self) -> &mut Vec<String>;
}

macro_rules! monitor_type {
    ($struct_name:ident $type:ident {
        $($field:tt)*
    }) => {
        #[serde_inline_default]
        #[skip_serializing_none]
        #[serde_as]
        #[derive(Clone, Debug, Derivative, Serialize, Deserialize)]
        #[derivative(PartialEq)]
        pub struct $struct_name {
            #[serde(rename = "id")]
            #[serde_as(as = "Option<DeserializeNumberLenient>")]
            pub id: Option<i32>,

            #[serde(rename = "name")]
            pub name: Option<String>,

            #[serde(rename = "interval")]
            #[serde_inline_default(Some(60))]
            #[serde_as(as = "Option<DeserializeNumberLenient>")]
            pub interval: Option<i32>,

            #[serde(rename = "active")]
            #[serde_inline_default(None)]
            #[serde_as(as = "Option<DeserializeBoolLenient>")]
            pub active: Option<bool>,

            #[serde(rename = "maxretries")]
            #[serde_as(as = "Option<DeserializeNumberLenient>")]
            pub max_retries: Option<i32>,

            #[serde(rename = "retryInterval")]
            #[serde_inline_default(Some(60))]
            #[serde_as(as = "Option<DeserializeNumberLenient>")]
            pub retry_interval: Option<i32>,

            #[serde(rename = "upsideDown")]
            #[serde_as(as = "Option<DeserializeBoolLenient>")]
            pub upside_down: Option<bool>,

            #[serde(rename = "parent")]
            #[serde_as(as = "Option<DeserializeNumberLenient>")]
            #[serialize_always]
            pub parent: Option<i32>,

            #[serde(rename = "parent_name")]
            #[derivative(PartialEq = "ignore")]
            #[derivative(Hash = "ignore")]
            pub parent_name: Option<String>,

            #[serde(rename = "tags")]
            #[serde(skip_serializing_if = "Vec::is_empty")]
            #[serde(default)]
            #[serde_as(as = "DeserializeVecLenient<Tag>")]
            #[derivative(PartialEq(compare_with = "compare_tags"))]
            pub tags: Vec<Tag>,

            #[serde(rename = "notificationIDList")]
            #[serde_as(as = "Option<DeserializeHashMapLenient<String, bool>>")]
            pub notification_id_list: Option<HashMap<String, bool>>,

            #[serde(rename = "accepted_statuscodes")]
            #[serde_inline_default(vec!["200-299".to_owned()])]
            pub accepted_statuscodes: Vec<String>,

            $($field)*
        }

        impl MonitorCommon for $struct_name {
            fn id(&self) -> &Option<i32> { &self.id }
            fn id_mut(&mut self) -> &mut Option<i32> { &mut self.id }
            fn name(&self) -> &Option<String> { &self.name }
            fn name_mut(&mut self) -> &mut Option<String> { &mut self.name }
            fn interval(&self) -> &Option<i32> { &self.interval }
            fn interval_mut(&mut self) -> &mut Option<i32> { &mut self.interval }
            fn active(&self) -> &Option<bool> { &self.active }
            fn active_mut(&mut self) -> &mut Option<bool> { &mut self.active }
            fn max_retries(&self) -> &Option<i32> { &self.max_retries }
            fn max_retries_mut(&mut self) -> &mut Option<i32> { &mut self.max_retries }
            fn retry_interval(&self) -> &Option<i32> { &self.retry_interval }
            fn retry_interval_mut(&mut self) -> &mut Option<i32> { &mut self.retry_interval }
            fn upside_down(&self) -> &Option<bool> { &self.upside_down }
            fn upside_down_mut(&mut self) -> &mut Option<bool> { &mut self.upside_down }
            fn parent(&self) -> &Option<i32> { &self.parent }
            fn parent_mut(&mut self) -> &mut Option<i32> { &mut self.parent }
            fn parent_name(&self) -> &Option<String> { &self.parent_name }
            fn parent_name_mut(&mut self) -> &mut Option<String> { &mut self.parent_name }
            fn tags(&self) -> &Vec<Tag> { &self.tags }
            fn tags_mut(&mut self) -> &mut Vec<Tag> { &mut self.tags }
            fn notification_id_list(&self) -> &Option<HashMap<String, bool>> { &self.notification_id_list }
            fn notification_id_list_mut(&mut self) -> &mut Option<HashMap<String, bool>> { &mut self.notification_id_list }
            fn accepted_statuscodes(&self) -> &Vec<String> { &self.accepted_statuscodes }
            fn accepted_statuscodes_mut(&mut self) -> &mut Vec<String> { &mut self.accepted_statuscodes }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                serde_json::from_value(json!({})).unwrap()
            }
        }

        impl $struct_name {
            pub fn new() -> Self {
                Default::default()
            }
        }

        impl From<$struct_name> for Monitor {
            fn from(value: $struct_name) -> Self {
                Monitor::$type { value: value }
            }
        }
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
    },

    #[serde(rename = "ntlm")]
    NTLM {
        #[serde(rename = "basic_auth_user")]
        basic_auth_user: Option<String>,

        #[serde(rename = "basic_auth_pass")]
        basic_auth_pass: Option<String>,

        #[serde(rename = "authDomain")]
        auth_domain: Option<String>,

        #[serde(rename = "authWorkstation")]
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

monitor_type! {
    MonitorGroup Group {

    }
}

monitor_type! {
    MonitorSqlServer SqlServer {
        #[serde(rename = "databaseConnectionString")]
        pub database_connection_string: Option<String>,
    }
}

monitor_type! {
    MonitorPostgres Postgres {
        #[serde(rename = "databaseConnectionString")]
        pub database_connection_string: Option<String>,
    }
}

monitor_type! {
    MonitorMongoDB Mongodb {
        #[serde(rename = "databaseConnectionString")]
        pub database_connection_string: Option<String>,
    }
}

monitor_type! {
    MonitorMysql Mysql {
        #[serde(rename = "databaseConnectionString")]
        pub database_connection_string: Option<String>,

        #[serde(rename = "radiusPassword")]
        pub password: Option<String>,
    }
}

monitor_type! {
    MonitorRedis Redis {
        #[serde(rename = "databaseConnectionString")]
        pub database_connection_string: Option<String>,
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
        pub port: Option<String>,

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
    }
}

monitor_type! {
    MonitorGameDig GameDig {
        #[serde(rename = "game")]
        pub game: Option<String>,

        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        pub port: Option<String>,

        #[serde(rename = "gamedigGivenPortOnly")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub gamedig_given_port_only: Option<bool>,
    }
}

monitor_type! {
    MonitorGrpcKeyword GrpcKeyword {
        #[serde(rename = "keyword")]
        pub keyword: Option<String>,

        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub invert_keyword: Option<bool>,

        #[serde(rename = "grpcUrl")]
        pub grpc_url: Option<String>,

        #[serde(rename = "maxredirects")]
        #[serde_inline_default(Some(10))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "grpcEnableTls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub grpc_enable_tls: Option<bool>,

        #[serde(rename = "grpcServiceName")]
        pub grpc_service_name: Option<String>,

        #[serde(rename = "grpcMethod")]
        pub grpc_method: Option<String>,

        #[serde(rename = "grpcProtobuf")]
        pub grpc_protobuf: Option<String>,

        #[serde(rename = "grpcBody")]
        pub grpc_body: Option<String>,

        #[serde(rename = "grpcMetadata")]
        pub grpc_metadata: Option<String>,
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

        #[serde(rename = "resendInterval")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub resend_interval: Option<i32>,

        #[serde(rename = "expiryNotification")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub expiry_notification: Option<bool>,

        #[serde(rename = "ignoreTls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub ignore_tls: Option<bool>,

        #[serde(rename = "maxredirects")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "proxyId")]
        pub proxy_id: Option<String>,

        #[serde(rename = "method")]
        #[serde_inline_default(Some(HttpMethod::GET))]
        pub method: Option<HttpMethod>,

        #[serde(rename = "httpBodyEncoding")]
        pub http_body_encoding: Option<String>,

        #[serde(rename = "body")]
        pub body: Option<String>,

        #[serde(rename = "headers")]
        pub headers: Option<String>,

        #[serde(flatten)]
        pub auth: Option<HttpAuth>,
    }
}

monitor_type! {
    MonitorJsonQuery JsonQuery {
        #[serde(rename = "jsonPath")]
        pub json_path: Option<String>,

        #[serde(rename = "expectedValue")]
        pub expected_value: Option<String>,

        #[serde(rename = "url")]
        pub url: Option<String>,

        #[serde(rename = "timeout")]
        #[serde_inline_default(Some(48))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub timeout: Option<i32>,

        #[serde(rename = "resendInterval")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub resend_interval: Option<i32>,

        #[serde(rename = "expiryNotification")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub expiry_notification: Option<bool>,

        #[serde(rename = "ignoreTls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub ignore_tls: Option<bool>,

        #[serde(rename = "maxredirects")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "proxyId")]
        pub proxy_id: Option<String>,

        #[serde(rename = "method")]
        #[serde_inline_default(Some(HttpMethod::GET))]
        pub method: Option<HttpMethod>,

        #[serde(rename = "httpBodyEncoding")]
        pub http_body_encoding: Option<String>,

        #[serde(rename = "body")]
        pub body: Option<String>,

        #[serde(rename = "headers")]
        pub headers: Option<String>,

        #[serde(flatten)]
        pub auth: Option<HttpAuth>,
    }
}

monitor_type! {
    MonitorKafkaProducer KafkaProducer {
        #[serde(rename = "kafkaProducerBrokers")]
        pub kafka_producer_brokers: Vec<String>,

        #[serde(rename = "kafkaProducerTopic")]
        pub kafka_producer_topic: Option<String>,

        #[serde(rename = "kafkaProducerMessage")]
        pub kafka_producer_message: Option<String>,

        #[serde(rename = "kafkaProducerSsl")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub kafka_producer_ssl: Option<bool>,

        #[serde(rename = "kafkaProducerAllowAutoTopicCreation")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub kafka_producer_allow_auto_topic_creation: Option<bool>,

        #[serde(rename = "kafkaProducerSaslOptions")]
        pub kafka_producer_sasl_options: Option<KafkaProducerSaslOptions>,
    }
}

monitor_type! {
    MonitorKeyword Keyword {
        #[serde(rename = "keyword")]
        pub keyword: Option<String>,

        #[serde(rename = "invertKeyword")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub invert_keyword: Option<bool>,

        #[serde(rename = "url")]
        pub url: Option<String>,

        #[serde(rename = "timeout")]
        #[serde_inline_default(Some(48))]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub timeout: Option<i32>,

        #[serde(rename = "resendInterval")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub resend_interval: Option<i32>,

        #[serde(rename = "expiryNotification")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub expiry_notification: Option<bool>,

        #[serde(rename = "ignoreTls")]
        #[serde_as(as = "Option<DeserializeBoolLenient>")]
        pub ignore_tls: Option<bool>,

        #[serde(rename = "maxredirects")]
        #[serde_as(as = "Option<DeserializeNumberLenient>")]
        pub max_redirects: Option<i32>,

        #[serde(rename = "proxyId")]
        pub proxy_id: Option<String>,

        #[serde(rename = "method")]
        #[serde_inline_default(Some(HttpMethod::GET))]
        pub method: Option<HttpMethod>,

        #[serde(rename = "httpBodyEncoding")]
        pub http_body_encoding: Option<String>,

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
        pub port: Option<String>,

        #[serde(rename = "mqttUsername")]
        pub mqtt_username: Option<String>,

        #[serde(rename = "mqttPassword")]
        pub mqtt_password: Option<String>,

        #[serde(rename = "mqttTopic")]
        pub mqtt_topic: Option<String>,

        #[serde(rename = "mqttCheckType")]
        pub mqtt_check_type: Option<String>,

        #[serde(rename = "mqttSuccessMessage")]
        pub mqtt_success_message: Option<String>,
    }
}

monitor_type! {
    MonitorPing Ping {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "packetSize")]
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
        pub port: Option<String>,
    }
}

monitor_type! {
    MonitorPush Push {
        #[serde(rename = "pushURL")]
        pub push_url: Option<String>,
    }
}

monitor_type! {
    MonitorRadius Radius {
        #[serde(rename = "hostname")]
        pub hostname: Option<String>,

        #[serde(rename = "port")]
        pub port: Option<String>,

        #[serde(rename = "radiusUsername")]
        pub radius_username: Option<String>,

        #[serde(rename = "radiusPassword")]
        pub radius_password: Option<String>,

        #[serde(rename = "radiusSecret")]
        pub radius_secret: Option<String>,

        #[serde(rename = "radiusCalledStationId")]
        pub radius_called_station_id: Option<String>,

        #[serde(rename = "radiusCallingStationId")]
        pub radius_calling_station_id: Option<String>,
    }
}

monitor_type! {
    MonitorRealBrowser RealBrowser {
        #[serde(rename = "rurl")]
        pub url: Option<String>,

        #[serde(rename = "remoteBrowsersToggle")]
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
        pub port: Option<String>,
    }
}

monitor_type! {
    MonitorTailscalePing TailscalePing {
        #[serde(rename = "hostname")]
        hostname: Option<String>,
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
            Monitor::Mqtt { .. } => MonitorType::Mqtt,
            Monitor::KafkaProducer { .. } => MonitorType::KafkaProducer,
            Monitor::SqlServer { .. } => MonitorType::SqlServer,
            Monitor::Postgres { .. } => MonitorType::Postgres,
            Monitor::Mysql { .. } => MonitorType::Mysql,
            Monitor::Mongodb { .. } => MonitorType::Mongodb,
            Monitor::Radius { .. } => MonitorType::Radius,
            Monitor::Redis { .. } => MonitorType::Redis,
            Monitor::TailscalePing { .. } => MonitorType::TailscalePing,
        }
    }

    pub fn common(&self) -> Box<&dyn MonitorCommon> {
        match self {
            Monitor::Group { value } => Box::new(value),
            Monitor::Http { value } => Box::new(value),
            Monitor::Port { value } => Box::new(value),
            Monitor::Ping { value } => Box::new(value),
            Monitor::Keyword { value } => Box::new(value),
            Monitor::JsonQuery { value } => Box::new(value),
            Monitor::GrpcKeyword { value } => Box::new(value),
            Monitor::Dns { value } => Box::new(value),
            Monitor::Docker { value } => Box::new(value),
            Monitor::RealBrowser { value } => Box::new(value),
            Monitor::Push { value } => Box::new(value),
            Monitor::Steam { value } => Box::new(value),
            Monitor::GameDig { value } => Box::new(value),
            Monitor::Mqtt { value } => Box::new(value),
            Monitor::KafkaProducer { value } => Box::new(value),
            Monitor::SqlServer { value } => Box::new(value),
            Monitor::Postgres { value } => Box::new(value),
            Monitor::Mysql { value } => Box::new(value),
            Monitor::Mongodb { value } => Box::new(value),
            Monitor::Radius { value } => Box::new(value),
            Monitor::Redis { value } => Box::new(value),
            Monitor::TailscalePing { value } => Box::new(value),
        }
    }

    pub fn common_mut(&mut self) -> Box<&mut dyn MonitorCommon> {
        match self {
            Monitor::Group { value } => Box::new(value),
            Monitor::Http { value } => Box::new(value),
            Monitor::Port { value } => Box::new(value),
            Monitor::Ping { value } => Box::new(value),
            Monitor::Keyword { value } => Box::new(value),
            Monitor::JsonQuery { value } => Box::new(value),
            Monitor::GrpcKeyword { value } => Box::new(value),
            Monitor::Dns { value } => Box::new(value),
            Monitor::Docker { value } => Box::new(value),
            Monitor::RealBrowser { value } => Box::new(value),
            Monitor::Push { value } => Box::new(value),
            Monitor::Steam { value } => Box::new(value),
            Monitor::GameDig { value } => Box::new(value),
            Monitor::Mqtt { value } => Box::new(value),
            Monitor::KafkaProducer { value } => Box::new(value),
            Monitor::SqlServer { value } => Box::new(value),
            Monitor::Postgres { value } => Box::new(value),
            Monitor::Mysql { value } => Box::new(value),
            Monitor::Mongodb { value } => Box::new(value),
            Monitor::Radius { value } => Box::new(value),
            Monitor::Redis { value } => Box::new(value),
            Monitor::TailscalePing { value } => Box::new(value),
        }
    }

    pub fn validate(&self, id: impl AsRef<str>) -> Result<()> {
        let mut errors = vec![];

        if self.common().name().is_none() {
            errors.push("Missing property 'name'".to_owned());
        }

        if !errors.is_empty() {
            return Err(Error::ValidationError(id.as_ref().to_owned(), errors));
        }

        Ok(())
    }
}

pub type MonitorList = HashMap<String, Monitor>;
