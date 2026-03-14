use itertools::Itertools;
use kuma_client::{
    docker_host::DockerHost,
    monitor::*,
    notification::Notification,
    status_page::StatusPage,
    tag::{Tag, TagDefinition},
};
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;
use serde_inline_default::serde_inline_default;
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use tera::Tera;
use thiserror::Error;
use unescaper::unescape;

const CONTAINER_TEMPLATE_JSON: &str = include_str!("../mock-data/container.json");
const SERVICE_TEMPLATE_JSON: &str = include_str!("../mock-data/service.json");
const SYSTEM_INFO_TEMPLATE_JSON: &str = include_str!("../mock-data/system_info.json");

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlaygroundError {
    #[error("Error while trying to parse labels: {0}")]
    LabelParseError(String),

    #[error("Unable to deserialize: {0}")]
    DeserializeError(String),

    #[error("Found invalid config '{0}': {1}")]
    InvalidConfig(String, String),

    #[error("Unable to parse compose file: {0}")]
    ComposeError(String),

    #[error("Unsupported label entry: {0}")]
    InvalidLabel(String),

    #[error("Referenced {0} named '{1}' was not found in the generated entities")]
    MissingReference(&'static str, String),

    #[error("{0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, PlaygroundError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFormat {
    Yaml,
    Toml,
    Json,
}

impl ConfigFormat {
    pub fn all() -> [Self; 3] {
        [Self::Yaml, Self::Toml, Self::Json]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Json => "json",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Yaml => "autokuma.yaml",
            Self::Toml => "autokuma.toml",
            Self::Json => "autokuma.json",
        }
    }
}

impl From<&str> for ConfigFormat {
    fn from(value: &str) -> Self {
        match value {
            "toml" => Self::Toml,
            "json" => Self::Json,
            _ => Self::Yaml,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    Container,
    Service,
}

impl TargetKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Container => "Container",
            Self::Service => "Service",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MockTarget {
    pub id: String,
    pub name: String,
    pub kind: TargetKind,
    pub labels: HashMap<String, String>,
    pub context_json: serde_json::Value,
}

impl MockTarget {
    pub fn context(&self) -> tera::Context {
        tera::Context::from_value(self.context_json.clone()).unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParsedEntity {
    pub id: String,
    pub entity_type: String,
    pub entity: Entity,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnippetOutput {
    pub rendered: String,
    pub extracted_labels: Vec<(String, String)>,
    pub entities: Vec<ParsedEntity>,
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundDockerConfig {
    #[serde_inline_default(true)]
    pub enabled: bool,

    #[serde_inline_default("kuma".to_owned())]
    pub label_prefix: String,
}

impl Default for PlaygroundDockerConfig {
    fn default() -> Self {
        serde_json::from_value(json!({})).unwrap()
    }
}

#[serde_alias(ScreamingSnakeCase)]
#[serde_inline_default]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaygroundConfig {
    #[serde_inline_default(PlaygroundDockerConfig::default())]
    pub docker: PlaygroundDockerConfig,

    #[serde_inline_default("".to_owned())]
    pub default_settings: String,

    #[serde_inline_default(HashMap::new())]
    pub snippets: HashMap<String, String>,

    #[serde_inline_default(false)]
    pub insecure_env_access: bool,
}

impl Default for PlaygroundConfig {
    fn default() -> Self {
        serde_json::from_value(json!({})).unwrap()
    }
}

#[derive(Clone, PartialEq)]
pub struct PlaygroundEngine {
    config: PlaygroundConfig,
    defaults: BTreeMap<String, Vec<(String, String)>>,
}

impl PlaygroundEngine {
    pub fn new(config: PlaygroundConfig) -> Result<Self> {
        let defaults = config
            .default_settings
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.split_once(':')
                    .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                    .ok_or_else(|| {
                        PlaygroundError::InvalidConfig(
                            "default_settings".to_owned(),
                            line.to_owned(),
                        )
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            config,
            defaults: group_by_prefix(defaults, "."),
        })
    }

    pub fn get_defaults(&self, entity_type: impl AsRef<str>) -> Vec<(String, serde_json::Value)> {
        vec![self.defaults.get("*"), self.defaults.get(entity_type.as_ref())]
            .into_iter()
            .flat_map(|defaults| defaults.into_iter().flatten())
            .map(|(key, value)| (key.to_owned(), json!(value)))
            .collect()
    }

    pub fn collect_compose_entities(&self, targets: &[MockTarget]) -> Result<Vec<ParsedEntity>> {
        let mut entities = Vec::new();

        for target in targets {
            let context = target.context();
            let kuma_labels = get_kuma_labels(self, Some(&target.labels), &context)?;
            entities.extend(parse_entities_from_labels(self, kuma_labels, &context)?);
        }

        resolve_names_locally(&mut entities, &[])?;

        Ok(entities)
    }

    #[cfg(test)]
    pub fn render_snippet(
        &self,
        target: &MockTarget,
        snippet: &str,
        compose_entities: &[ParsedEntity],
    ) -> Result<SnippetOutput> {
        let context = target.context();
        let rendered = self.render_template(target, snippet)?;
        let extracted_labels = parse_snippet_lines(&rendered)?;
        let mut entities = parse_entities_from_labels(self, extracted_labels.clone(), &context)?;
        resolve_names_locally(&mut entities, compose_entities)?;

        Ok(SnippetOutput {
            rendered,
            extracted_labels,
            entities,
        })
    }

    pub fn render_template(&self, target: &MockTarget, template: &str) -> Result<String> {
        let context = target.context();
        let rendered = fill_templates(self, template, &context)?;
        Ok(rendered
            .trim_matches(|ch| ch == '\n' || ch == '\r')
            .to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "EntityWrapper", into = "EntityWrapper")]
pub enum Entity {
    DockerHost(DockerHost),
    Notification(Notification),
    Monitor(Monitor),
    Tag(TagDefinition),
    StatusPage(StatusPage),
}

impl From<DockerHost> for Entity {
    fn from(value: DockerHost) -> Self {
        Self::DockerHost(value)
    }
}

impl From<Notification> for Entity {
    fn from(value: Notification) -> Self {
        Self::Notification(value)
    }
}

impl From<Monitor> for Entity {
    fn from(value: Monitor) -> Self {
        Self::Monitor(value)
    }
}

impl From<TagDefinition> for Entity {
    fn from(value: TagDefinition) -> Self {
        Self::Tag(value)
    }
}

impl From<StatusPage> for Entity {
    fn from(value: StatusPage) -> Self {
        Self::StatusPage(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    #[serde(rename = "docker_host")]
    DockerHost,
    #[serde(rename = "notification")]
    Notification,
    #[serde(rename = "tag")]
    Tag,
    #[serde(rename = "status_page")]
    StatusPage,
    #[serde(untagged)]
    Monitor(MonitorType),
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_value(self)
            .map_err(|_| std::fmt::Error)?
            .as_str()
            .ok_or(std::fmt::Error)?
            .fmt(f)
    }
}

impl Entity {
    pub fn entity_type(&self) -> EntityType {
        match self {
            Entity::DockerHost(_) => EntityType::DockerHost,
            Entity::Notification(_) => EntityType::Notification,
            Entity::Monitor(monitor) => EntityType::Monitor(monitor.monitor_type()),
            Entity::Tag(_) => EntityType::Tag,
            Entity::StatusPage(_) => EntityType::StatusPage,
        }
    }
}

trait ParseValue {
    fn parse_type<T>(value: &serde_json::Value) -> Result<T>
    where
        T: Sized + serde::de::DeserializeOwned,
    {
        let entity_type = value
            .as_object()
            .ok_or_else(|| PlaygroundError::DeserializeError("Invalid entity structure".to_owned()))?
            .get("type")
            .ok_or_else(|| PlaygroundError::DeserializeError("Missing `type` parameter".to_owned()))?
            .as_str()
            .ok_or_else(|| {
                PlaygroundError::DeserializeError(
                    "Invalid `type` parameter (expected string)".to_owned(),
                )
            })?
            .to_owned();

        serde_json::from_value::<T>(serde_json::Value::String(entity_type))
            .map_err(|_| PlaygroundError::DeserializeError("Invalid `type` parameter".to_owned()))
    }

    fn parse(value: serde_json::Value) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! parse_entity {
    ($result_type:ty, $entity_type:ty, $value:expr) => {
        serde_json::from_value::<$entity_type>($value).map(|v| <$result_type>::from(v).into())
    };
}

impl ParseValue for Monitor {
    fn parse(v: serde_json::Value) -> Result<Self> {
        match Self::parse_type::<MonitorType>(&v)? {
            MonitorType::Dns => parse_entity!(Monitor, MonitorDns, v),
            MonitorType::Docker => parse_entity!(Monitor, MonitorDocker, v),
            MonitorType::GameDig => parse_entity!(Monitor, MonitorGameDig, v),
            MonitorType::GlobalPing => parse_entity!(Monitor, MonitorGlobalPingWrapper, v),
            MonitorType::Group => parse_entity!(Monitor, MonitorGroup, v),
            MonitorType::GrpcKeyword => parse_entity!(Monitor, MonitorGrpcKeyword, v),
            MonitorType::Http => parse_entity!(Monitor, MonitorHttp, v),
            MonitorType::JsonQuery => parse_entity!(Monitor, MonitorJsonQuery, v),
            MonitorType::KafkaProducer => parse_entity!(Monitor, MonitorKafkaProducer, v),
            MonitorType::Keyword => parse_entity!(Monitor, MonitorKeyword, v),
            MonitorType::Mongodb => parse_entity!(Monitor, MonitorMongoDB, v),
            MonitorType::Mqtt => parse_entity!(Monitor, MonitorMqtt, v),
            MonitorType::Mysql => parse_entity!(Monitor, MonitorMysql, v),
            MonitorType::Ping => parse_entity!(Monitor, MonitorPing, v),
            MonitorType::Port => parse_entity!(Monitor, MonitorPort, v),
            MonitorType::Postgres => parse_entity!(Monitor, MonitorPostgres, v),
            MonitorType::Push => parse_entity!(Monitor, MonitorPush, v),
            MonitorType::Radius => parse_entity!(Monitor, MonitorRadius, v),
            MonitorType::RealBrowser => parse_entity!(Monitor, MonitorRealBrowser, v),
            MonitorType::Redis => parse_entity!(Monitor, MonitorRedis, v),
            MonitorType::Steam => parse_entity!(Monitor, MonitorSteam, v),
            MonitorType::SqlServer => parse_entity!(Monitor, MonitorSqlServer, v),
            MonitorType::TailscalePing => parse_entity!(Monitor, MonitorTailscalePing, v),
            #[cfg(not(feature = "uptime-kuma-v1"))]
            MonitorType::SMTP => parse_entity!(Monitor, MonitorSMTP, v),
            #[cfg(not(feature = "uptime-kuma-v1"))]
            MonitorType::SNMP => parse_entity!(Monitor, MonitorSNMP, v),
            #[cfg(not(feature = "uptime-kuma-v1"))]
            MonitorType::RabbitMQ => parse_entity!(Monitor, MonitorRabbitMQ, v),
        }
        .map_err(|e| PlaygroundError::LabelParseError(e.to_string()))
    }
}

impl ParseValue for Entity {
    fn parse(value: serde_json::Value) -> Result<Self> {
        match Self::parse_type::<EntityType>(&value)? {
            EntityType::DockerHost => parse_entity!(Entity, DockerHost, value),
            EntityType::Notification => parse_entity!(Entity, Notification, value),
            EntityType::Tag => parse_entity!(Entity, TagDefinition, value),
            EntityType::StatusPage => parse_entity!(Entity, StatusPage, value),
            EntityType::Monitor(_) => Ok(Monitor::parse(value).map(|v| v.into())?),
        }
        .map_err(|e| PlaygroundError::LabelParseError(e.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum EntityWrapper {
    DockerHost {
        #[serde(flatten)]
        docker_host: DockerHostTagged,
    },
    Notification {
        #[serde(flatten)]
        notification: NotificationTagged,
    },
    Tag {
        #[serde(flatten)]
        tag: TagTagged,
    },
    Monitor {
        #[serde(flatten)]
        monitor: Monitor,
    },
    StatusPage {
        #[serde(flatten)]
        status_page: StatusPageTagged,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum NotificationTagged {
    #[serde(rename = "notification")]
    Notification {
        #[serde(flatten)]
        notification: Notification,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum DockerHostTagged {
    #[serde(rename = "docker_host")]
    DockerHost {
        #[serde(flatten)]
        docker_host: DockerHost,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TagTagged {
    #[serde(rename = "tag")]
    Tag {
        #[serde(flatten)]
        tag: TagDefinition,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum StatusPageTagged {
    #[serde(rename = "status_page")]
    StatusPage {
        #[serde(flatten)]
        status_page: StatusPage,
    },
}

impl From<EntityWrapper> for Entity {
    fn from(wrapper: EntityWrapper) -> Self {
        match wrapper {
            EntityWrapper::DockerHost {
                docker_host: DockerHostTagged::DockerHost { docker_host },
            } => Entity::DockerHost(docker_host),
            EntityWrapper::Notification {
                notification: NotificationTagged::Notification { notification },
            } => Entity::Notification(notification),
            EntityWrapper::Tag {
                tag: TagTagged::Tag { tag },
            } => Entity::Tag(tag),
            EntityWrapper::StatusPage {
                status_page: StatusPageTagged::StatusPage { status_page },
            } => Entity::StatusPage(status_page),
            EntityWrapper::Monitor { monitor } => Entity::Monitor(monitor),
        }
    }
}

impl From<Entity> for EntityWrapper {
    fn from(entity: Entity) -> Self {
        match entity {
            Entity::DockerHost(docker_host) => EntityWrapper::DockerHost {
                docker_host: DockerHostTagged::DockerHost { docker_host },
            },
            Entity::Notification(notification) => EntityWrapper::Notification {
                notification: NotificationTagged::Notification { notification },
            },
            Entity::Tag(tag) => EntityWrapper::Tag {
                tag: TagTagged::Tag { tag },
            },
            Entity::StatusPage(status_page) => EntityWrapper::StatusPage {
                status_page: StatusPageTagged::StatusPage { status_page },
            },
            Entity::Monitor(monitor) => EntityWrapper::Monitor { monitor },
        }
    }
}

pub fn parse_config(text: &str, format: &ConfigFormat) -> Result<PlaygroundConfig> {
    if text.trim().is_empty() {
        return Ok(PlaygroundConfig::default());
    }

    match format {
        ConfigFormat::Yaml => {
            serde_yaml::from_str(text).map_err(|e| PlaygroundError::DeserializeError(e.to_string()))
        }
        ConfigFormat::Toml => {
            toml::from_str(text).map_err(|e| PlaygroundError::DeserializeError(e.to_string()))
        }
        ConfigFormat::Json => {
            serde_json::from_str(text).map_err(|e| PlaygroundError::DeserializeError(e.to_string()))
        }
    }
}

pub fn parse_compose_targets(compose_text: &str) -> Result<Vec<MockTarget>> {
    if compose_text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let compose: ComposeFile =
        serde_yaml::from_str(compose_text).map_err(|e| PlaygroundError::ComposeError(e.to_string()))?;

    let system_info = build_system_info(&compose);
    let mut targets = Vec::new();

    for (index, (service_name, service)) in compose
        .services
        .into_iter()
        .sorted_by(|a, b| a.0.cmp(&b.0))
        .enumerate()
    {
        let container_name = service
            .container_name
            .clone()
            .unwrap_or_else(|| service_name.clone());
        let container_labels = normalize_labels(service.labels.clone())?;
        let service_labels = normalize_labels(service.deploy.as_ref().and_then(|deploy| deploy.labels.clone()))?;
        let image = service.image.clone().unwrap_or_else(|| "docker.io/library/alpine:latest".to_owned());
        let container_id = pseudo_digest(&format!("container:{service_name}"), 64);
        let image_id = format!("sha256:{}", pseudo_digest(&format!("image:{image}"), 64));
        let container = build_container_value(
            index,
            &service_name,
            &container_name,
            &image,
            &image_id,
            &container_id,
            &container_labels,
            &service,
        );
        let service_struct = build_service_value(index, &service_name, &image, &service_labels, &service);

        let container_context = json!({
            "container_id": container_id,
            "image_id": image_id,
            "image": image,
            "container_name": container_name,
            "container": container,
            "system_info": system_info.clone(),
        });

        let service_context = json!({
            "service": service_struct,
            "system_info": system_info.clone(),
        });

        targets.push(MockTarget {
            id: format!("container:{service_name}"),
            name: container_name,
            kind: TargetKind::Container,
            labels: container_labels,
            context_json: container_context,
        });

        targets.push(MockTarget {
            id: format!("service:{service_name}"),
            name: service_name.clone(),
            kind: TargetKind::Service,
            labels: service_labels,
            context_json: service_context,
        });
    }

    Ok(targets)
}

pub fn get_kuma_labels(
    engine: &PlaygroundEngine,
    labels: Option<&HashMap<String, String>>,
    template_values: &tera::Context,
) -> Result<Vec<(String, String)>> {
    labels.map_or_else(
        || Ok(vec![]),
        |labels| {
            labels
                .iter()
                .filter(|(key, _)| {
                    key.starts_with(&format!("{}.", engine.config.docker.label_prefix))
                })
                .map(|(key, value)| {
                    fill_templates(
                        engine,
                        key.trim_start_matches(&format!("{}.", engine.config.docker.label_prefix)),
                        template_values,
                    )
                    .map(|key| (key, value.to_owned()))
                })
                .chain(
                    labels
                        .iter()
                        .filter(|(key, _)| engine.config.snippets.contains_key(&format!("!{}", key)))
                        .map(|(key, value)| Ok((format!("__!{}", key), value.to_owned()))),
                )
                .collect::<Result<Vec<_>>>()
        },
    )
}

pub fn fill_templates(
    engine: &PlaygroundEngine,
    template: impl Into<String>,
    template_values: &tera::Context,
) -> Result<String> {
    let template = template.into();
    let mut tera = Tera::default();
    let allow_env = engine.config.insecure_env_access;

    tera.register_function(
        "get_env",
        move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
            let name = match args.get("name") {
                Some(val) => tera::from_value::<String>(val.clone()).map_err(|_| {
                    tera::Error::msg(format!(
                        "Function `get_env` received name={} but `name` can only be a string",
                        val
                    ))
                })?,
                None => {
                    return Err(tera::Error::msg(
                        "Function `get_env` didn't receive a `name` argument",
                    ));
                }
            };

            if !allow_env && !name.starts_with("AUTOKUMA__ENV__") {
                return Err(tera::Error::msg(format!(
                    "Access to environment variable `{}` is not allowed",
                    &name
                )));
            }

            match std::env::var(&name).ok() {
                Some(res) => Ok(tera::Value::String(res)),
                None => match args.get("default") {
                    Some(default) => Ok(default.clone()),
                    None => Err(tera::Error::msg(format!(
                        "Environment variable `{}` not found",
                        &name
                    ))),
                },
            }
        },
    );

    tera.add_raw_template(&template, &template)
        .and_then(|_| tera.render(&template, template_values))
        .map_err(|e| PlaygroundError::LabelParseError(print_error_chain(&e)))
}

fn print_error_chain(error: &dyn std::error::Error) -> String {
    let mut result = "\n".to_owned();
    let mut current_error = Some(error);

    while let Some(err) = current_error {
        result.push_str(&format!("Caused by: {}\n", err));
        current_error = err.source();
    }

    result
}

fn parse_snippet_lines(rendered: &str) -> Result<Vec<(String, String)>> {
    rendered
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split_once(": ")
                .map(|(key, value)| {
                    (
                        key.trim_start().to_owned(),
                        unescape(value).unwrap_or_else(|_| value.to_owned()),
                    )
                })
                .ok_or_else(|| PlaygroundError::InvalidLabel(line.to_owned()))
        })
        .collect()
}

fn parse_entities_from_labels(
    engine: &PlaygroundEngine,
    labels: Vec<(String, String)>,
    template_values: &tera::Context,
) -> Result<Vec<ParsedEntity>> {
    let entries = labels
        .iter()
        .flat_map(|(key, value)| {
            if key.starts_with("__") {
                let snippet = engine
                    .config
                    .snippets
                    .get(key.trim_start_matches("__"))
                    .cloned();

                let args = if key.starts_with("__!") {
                    Some(vec![serde_json::Value::String(value.to_owned())])
                } else {
                    serde_json::from_str::<Vec<serde_json::Value>>(&format!("[{}]", value)).ok()
                };

                if let (Some(snippet), Some(args)) = (snippet, args) {
                    let mut nested_context = template_values.clone();
                    nested_context.insert("args", &args);

                    if let Ok(snippet) = fill_templates(engine, snippet, &nested_context) {
                        parse_snippet_lines(&snippet).unwrap_or_default()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                vec![(key.to_owned(), value.to_owned())]
            }
        })
        .collect::<Vec<_>>();

    group_by_prefix(entries, ".")
        .into_iter()
        .map(|(key, value)| (key, group_by_prefix(value, ".")))
        .flat_map(|(id, entities)| {
            entities.into_iter().map(move |(prefix, settings)| {
                let entity_type = serde_json::from_value::<EntityType>(serde_json::Value::String(prefix))
                    .map_err(|_| {
                        PlaygroundError::DeserializeError(format!(
                            "Cannot create {id} because it has an invalid type"
                        ))
                    })?;

                let entity = get_entity_from_settings(
                    engine,
                    &id,
                    &entity_type,
                    settings
                        .into_iter()
                        .map(|(key, value)| (key, json!(value)))
                        .collect_vec(),
                    template_values,
                )?;

                Ok(ParsedEntity {
                    id: id.clone(),
                    entity_type: entity.entity_type().to_string(),
                    entity,
                })
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn resolve_names_locally(
    entities: &mut [ParsedEntity],
    references: &[ParsedEntity],
) -> Result<()> {
    let monitor_ids = assign_ids(
        references,
        entities,
        |entity| matches!(entity, Entity::Monitor(_)),
        |entity, id| {
            if let Entity::Monitor(monitor) = entity {
                *monitor.common_mut().id_mut() = Some(id);
            }
        },
    );
    let notification_ids = assign_ids(
        references,
        entities,
        |entity| matches!(entity, Entity::Notification(_)),
        |entity, id| {
            if let Entity::Notification(notification) = entity {
                notification.id = Some(id);
            }
        },
    );
    let docker_host_ids = assign_ids(
        references,
        entities,
        |entity| matches!(entity, Entity::DockerHost(_)),
        |entity, id| {
            if let Entity::DockerHost(host) = entity {
                host.id = Some(id);
            }
        },
    );
    let tag_ids = assign_ids(
        references,
        entities,
        |entity| matches!(entity, Entity::Tag(_)),
        |entity, id| {
            if let Entity::Tag(tag) = entity {
                tag.tag_id = Some(id);
            }
        },
    );

    for parsed in entities.iter_mut() {
        let Entity::Monitor(monitor) = &mut parsed.entity else {
            continue;
        };

        if let Some(group_name) = monitor.common().parent_name().clone() {
            let group_id = monitor_ids
                .get(&group_name)
                .copied()
                .ok_or_else(|| PlaygroundError::MissingReference("monitor", group_name.clone()))?;
            *monitor.common_mut().parent_mut() = Some(group_id);
        }

        if let Some(notification_names) = monitor.common().notification_names().clone() {
            let notification_id_list = notification_names
                .into_iter()
                .map(|name| {
                    notification_ids
                        .get(&name)
                        .copied()
                        .map(|id| (id.to_string(), true))
                        .ok_or_else(|| PlaygroundError::MissingReference("notification", name))
                })
                .collect::<Result<HashMap<String, bool>>>()?;

            monitor
                .common_mut()
                .notification_id_list_mut()
                .get_or_insert(HashMap::new())
                .extend(notification_id_list);
        }

        if let Some(tag_names) = monitor.common().tag_names().clone() {
            let mut tags = tag_names
                .into_iter()
                .map(|tag_value| {
                    let id = tag_ids
                        .get(&tag_value.name)
                        .copied()
                        .ok_or_else(|| {
                            PlaygroundError::MissingReference("tag", tag_value.name.clone())
                        })?;

                    Ok(Tag {
                        tag_id: Some(id),
                        name: None,
                        value: tag_value.value,
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<Tag>>>()?;

            monitor.common_mut().tags_mut().append(&mut tags);
        }

        if let Monitor::Docker { value } = monitor {
            if let Some(docker_host_name) = value.docker_host_name.clone() {
                let docker_host_id = docker_host_ids
                    .get(&docker_host_name)
                    .copied()
                    .ok_or_else(|| {
                        PlaygroundError::MissingReference("docker_host", docker_host_name)
                    })?;

                value.docker_host = Some(docker_host_id);
            }
        }
    }

    Ok(())
}

fn assign_ids(
    references: &[ParsedEntity],
    entities: &mut [ParsedEntity],
    predicate: impl Fn(&Entity) -> bool,
    mut assign: impl FnMut(&mut Entity, i32),
) -> HashMap<String, i32> {
    let mut next_id = 1;
    let mut ids = HashMap::new();
    let mut seen = HashSet::new();

    for parsed in references.iter().chain(entities.iter()) {
        if predicate(&parsed.entity) && seen.insert(parsed.id.clone()) {
            ids.insert(parsed.id.clone(), next_id);
            next_id += 1;
        }
    }

    for parsed in entities.iter_mut() {
        if predicate(&parsed.entity) {
            let assigned_id = *ids.get(&parsed.id).unwrap_or(&next_id);
            assign(&mut parsed.entity, assigned_id);
        }
    }

    ids
}

fn get_entity_from_settings(
    engine: &PlaygroundEngine,
    id: &str,
    entity_type: &EntityType,
    settings: Vec<(String, serde_json::Value)>,
    context: &tera::Context,
) -> Result<Entity> {
    let defaults = engine.get_defaults(entity_type.to_string());

    let config = fill_templates(
        engine,
        vec![("type".to_owned(), json!(entity_type.to_owned()))]
            .into_iter()
            .chain(settings)
            .chain(defaults)
            .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
            .unique_by(|(key, _)| key.to_owned())
            .map(|(key, value)| match value {
                serde_json::Value::String(s) if s.starts_with('"') && s.ends_with('"') => {
                    format!("{key} = {s}")
                }
                other => format!("{key} = {other}"),
            })
            .join("\n"),
        context,
    )?;

    let toml = toml::from_str::<serde_json::Value>(&config)
        .map_err(|e| PlaygroundError::LabelParseError(e.to_string()))?;

    let entity = Entity::parse(toml)?;

    if let Entity::Monitor(monitor) = &entity {
        monitor
            .validate(id)
            .map_err(|e| PlaygroundError::ValidationError(e.to_string()))?;
    }

    Ok(entity)
}

fn group_by_prefix(
    values: impl IntoIterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
    delimiter: &str,
) -> BTreeMap<String, Vec<(String, String)>> {
    values.into_iter().fold(BTreeMap::new(), |mut groups, (key, value)| {
        if let Some((prefix, key)) = key.as_ref().split_once(delimiter) {
            groups
                .entry(prefix.to_owned())
                .or_default()
                .push((key.to_owned(), value.as_ref().to_owned()));
        }
        groups
    })
}

#[derive(Clone, Debug, Deserialize)]
struct ComposeFile {
    #[serde(default)]
    services: BTreeMap<String, ComposeService>,
}

#[derive(Clone, Debug, Deserialize)]
struct ComposeService {
    image: Option<String>,
    container_name: Option<String>,
    command: Option<ComposeCommand>,
    entrypoint: Option<ComposeCommand>,
    hostname: Option<String>,
    domainname: Option<String>,
    labels: Option<ComposeLabels>,
    networks: Option<ComposeNetworks>,
    ports: Option<Vec<ComposePort>>, 
    expose: Option<Vec<ComposeExpose>>,
    volumes: Option<Vec<ComposeVolume>>,
    deploy: Option<ComposeDeploy>,
}

#[derive(Clone, Debug, Deserialize)]
struct ComposeDeploy {
    labels: Option<ComposeLabels>,
    replicas: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ComposeLabels {
    Map(BTreeMap<String, serde_json::Value>),
    List(Vec<String>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ComposeCommand {
    String(String),
    List(Vec<String>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ComposeNetworks {
    List(Vec<String>),
    Map(BTreeMap<String, ComposeNetworkConfig>),
}

#[derive(Clone, Debug, Default, Deserialize)]
struct ComposeNetworkConfig {
    aliases: Option<Vec<String>>,
    ipv4_address: Option<String>,
    ipv6_address: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ComposePort {
    Short(String),
    Long(ComposePortConfig),
}

#[derive(Clone, Debug, Deserialize)]
struct ComposePortConfig {
    target: u16,
    published: Option<u16>,
    protocol: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ComposeExpose {
    Number(u16),
    String(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ComposeVolume {
    Short(String),
    Long(ComposeVolumeConfig),
}

#[derive(Clone, Debug, Deserialize)]
struct ComposeVolumeConfig {
    #[serde(rename = "type")]
    volume_type: Option<String>,
    source: Option<String>,
    target: Option<String>,
    read_only: Option<bool>,
}

fn normalize_labels(labels: Option<ComposeLabels>) -> Result<HashMap<String, String>> {
    match labels {
        None => Ok(HashMap::new()),
        Some(ComposeLabels::Map(map)) => Ok(map
            .into_iter()
            .map(|(key, value)| {
                let value = match value {
                    serde_json::Value::String(value) => value,
                    other => other.to_string(),
                };
                (key, value)
            })
            .collect()),
        Some(ComposeLabels::List(entries)) => entries
            .into_iter()
            .map(|entry| {
                entry
                    .split_once('=')
                    .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                    .ok_or(PlaygroundError::InvalidLabel(entry))
            })
            .collect(),
    }
}

fn build_system_info(compose: &ComposeFile) -> serde_json::Value {
    let mut system_info = template_json(SYSTEM_INFO_TEMPLATE_JSON);
    let image_count = compose
        .services
        .values()
        .filter_map(|service| service.image.as_ref())
        .collect::<HashSet<_>>()
        .len();
    let service_count = compose.services.len();

    if let Some(obj) = system_info.as_object_mut() {
        obj.insert("Containers".to_owned(), json!(service_count));
        obj.insert("ContainersRunning".to_owned(), json!(service_count));
        obj.insert("ContainersPaused".to_owned(), json!(0));
        obj.insert("ContainersStopped".to_owned(), json!(0));
        obj.insert("Images".to_owned(), json!(image_count));
        obj.insert("Name".to_owned(), json!("autokuma-playground"));
        obj.insert("OperatingSystem".to_owned(), json!("Docker Compose Mock Host"));
        obj.insert("OSVersion".to_owned(), json!("playground"));
        obj.insert("Labels".to_owned(), json!([
            "orchestration=docker-compose",
            format!("services={service_count}"),
        ]));
        obj.insert("ServerVersion".to_owned(), json!("compose-mock"));
    }

    system_info
}

fn build_container_value(
    index: usize,
    service_name: &str,
    container_name: &str,
    image: &str,
    image_id: &str,
    container_id: &str,
    labels: &HashMap<String, String>,
    service: &ComposeService,
) -> serde_json::Value {
    let mut container = template_json(CONTAINER_TEMPLATE_JSON);
    let ports = build_container_ports(service);
    let mounts = build_mounts(service);
    let networks = build_container_networks(index, service_name, container_name, service);
    let command = service
        .command
        .as_ref()
        .or(service.entrypoint.as_ref())
        .map(command_to_string)
        .unwrap_or_else(|| "/bin/sh".to_owned());

    if let Some(obj) = container.as_object_mut() {
        obj.insert("Id".to_owned(), json!(container_id));
        obj.insert("Names".to_owned(), json!([format!("/{container_name}")]));
        obj.insert("Image".to_owned(), json!(image));
        obj.insert("ImageID".to_owned(), json!(image_id));
        obj.insert("Command".to_owned(), json!(command));
        obj.insert("Created".to_owned(), json!((1739811096 + index as i64).to_string()));
        obj.insert("Labels".to_owned(), json!(labels));
        obj.insert("State".to_owned(), json!("running"));
        obj.insert("Status".to_owned(), json!("Up 1 minute"));
        obj.insert("Ports".to_owned(), serde_json::Value::Array(ports));
        obj.insert("Mounts".to_owned(), serde_json::Value::Array(mounts));
        if let Some(host_config) = obj.get_mut("HostConfig") {
            if let Some(host_obj) = host_config.as_object_mut() {
                host_obj.insert(
                    "NetworkMode".to_owned(),
                    json!(first_network_name(service).unwrap_or("default")),
                );
            }
        }
        if let Some(network_settings) = obj.get_mut("NetworkSettings") {
            if let Some(network_obj) = network_settings.as_object_mut() {
                network_obj.insert("Networks".to_owned(), networks);
            }
        }
    }

    container
}

fn build_service_value(
    index: usize,
    service_name: &str,
    image: &str,
    labels: &HashMap<String, String>,
    service: &ComposeService,
) -> serde_json::Value {
    let mut service_value = template_json(SERVICE_TEMPLATE_JSON);
    let ports = build_service_ports(service);
    let virtual_ips = build_virtual_ips(index, service);
    let replicas = service
        .deploy
        .as_ref()
        .and_then(|deploy| deploy.replicas)
        .unwrap_or(1);

    if let Some(obj) = service_value.as_object_mut() {
        obj.insert("ID".to_owned(), json!(pseudo_digest(&format!("service:{service_name}"), 25)));
        if let Some(spec) = obj.get_mut("Spec").and_then(|value| value.as_object_mut()) {
            spec.insert("Name".to_owned(), json!(service_name));
            spec.insert("Labels".to_owned(), json!(labels));
            if let Some(task_template) = spec
                .get_mut("TaskTemplate")
                .and_then(|value| value.as_object_mut())
            {
                if let Some(container_spec) = task_template
                    .get_mut("ContainerSpec")
                    .and_then(|value| value.as_object_mut())
                {
                    container_spec.insert("Image".to_owned(), json!(image));
                    if !labels.is_empty() {
                        container_spec.insert("Labels".to_owned(), json!(labels));
                    }
                }
            }
            if let Some(mode) = spec.get_mut("Mode").and_then(|value| value.as_object_mut()) {
                mode.insert("Replicated".to_owned(), json!({ "Replicas": replicas }));
            }
            if let Some(endpoint_spec) = spec
                .get_mut("EndpointSpec")
                .and_then(|value| value.as_object_mut())
            {
                endpoint_spec.insert("Ports".to_owned(), serde_json::Value::Array(ports.clone()));
            }
        }
        if let Some(endpoint) = obj.get_mut("Endpoint").and_then(|value| value.as_object_mut()) {
            if let Some(endpoint_spec) = endpoint
                .get_mut("Spec")
                .and_then(|value| value.as_object_mut())
            {
                endpoint_spec.insert("Ports".to_owned(), serde_json::Value::Array(ports.clone()));
            }
            endpoint.insert("Ports".to_owned(), serde_json::Value::Array(ports));
            endpoint.insert("VirtualIPs".to_owned(), serde_json::Value::Array(virtual_ips));
        }
    }

    service_value
}

fn template_json(template: &str) -> serde_json::Value {
    serde_json::from_str(template).expect("mock-data json must stay valid")
}

fn command_to_string(command: &ComposeCommand) -> String {
    match command {
        ComposeCommand::String(value) => value.clone(),
        ComposeCommand::List(values) => values.join(" "),
    }
}

fn build_container_ports(service: &ComposeService) -> Vec<serde_json::Value> {
    let mut ports = service
        .ports
        .as_ref()
        .into_iter()
        .flatten()
        .filter_map(parse_port_mapping)
        .map(|port| {
            json!({
                "PrivatePort": port.target,
                "PublicPort": port.published,
                "Type": port.protocol,
            })
        })
        .collect::<Vec<_>>();

    if ports.is_empty() {
        ports.extend(
            service
                .expose
                .as_ref()
                .into_iter()
                .flatten()
                .filter_map(expose_to_port)
                .map(|port| {
                    json!({
                        "PrivatePort": port,
                        "Type": "tcp",
                    })
                }),
        );
    }

    ports
}

fn build_service_ports(service: &ComposeService) -> Vec<serde_json::Value> {
    service
        .ports
        .as_ref()
        .into_iter()
        .flatten()
        .filter_map(parse_port_mapping)
        .map(|port| {
            json!({
                "Protocol": port.protocol,
                "TargetPort": port.target,
                "PublishedPort": port.published.unwrap_or(port.target),
            })
        })
        .collect()
}

fn build_mounts(service: &ComposeService) -> Vec<serde_json::Value> {
    service
        .volumes
        .as_ref()
        .into_iter()
        .flatten()
        .filter_map(parse_volume)
        .map(|mount| {
            json!({
                "Type": mount.mount_type,
                "Name": mount.name,
                "Source": mount.source,
                "Destination": mount.destination,
                "Driver": mount.driver,
                "Mode": mount.mode,
                "RW": mount.rw,
                "Propagation": "",
            })
        })
        .collect()
}

fn build_container_networks(
    index: usize,
    service_name: &str,
    container_name: &str,
    service: &ComposeService,
) -> serde_json::Value {
    let network_template = template_json(CONTAINER_TEMPLATE_JSON)
        .get("NetworkSettings")
        .and_then(|value| value.get("Networks"))
        .and_then(|value| value.get("property1"))
        .cloned()
        .unwrap_or_else(|| json!({}));

    let mut networks = serde_json::Map::new();
    let host_alias = service.hostname.clone();
    let fqdn_alias = service.hostname.as_ref().map(|hostname| match service.domainname.as_ref() {
        Some(domainname) => format!("{hostname}.{domainname}"),
        None => hostname.clone(),
    });
    for (network_index, network) in iterate_networks(service).into_iter().enumerate() {
        let mut value = network_template.clone();
        if let Some(obj) = value.as_object_mut() {
            let aliases = network
                .aliases
                .iter()
                .cloned()
                .chain(host_alias.clone())
                .chain(fqdn_alias.clone())
                .unique()
                .collect::<Vec<_>>();
            obj.insert(
                "Aliases".to_owned(),
                json!(aliases),
            );
            obj.insert("NetworkID".to_owned(), json!(pseudo_digest(&format!("network:{}", network.name), 64)));
            obj.insert("EndpointID".to_owned(), json!(pseudo_digest(&format!("endpoint:{service_name}:{}", network.name), 64)));
            obj.insert("IPAddress".to_owned(), json!(network.ipv4_address.unwrap_or_else(|| fake_ipv4(index + network_index))));
            obj.insert("GlobalIPv6Address".to_owned(), json!(network.ipv6_address.unwrap_or_else(|| fake_ipv6(index + network_index))));
            obj.insert(
                "DNSNames".to_owned(),
                json!(aliases
                    .iter()
                    .cloned()
                    .chain([container_name.to_owned(), service_name.to_owned()])
                    .unique()
                    .collect::<Vec<_>>()),
            );
        }
        networks.insert(network.name, value);
    }

    serde_json::Value::Object(networks)
}

fn build_virtual_ips(index: usize, service: &ComposeService) -> Vec<serde_json::Value> {
    iterate_networks(service)
        .into_iter()
        .enumerate()
        .map(|(network_index, network)| {
            json!({
                "NetworkID": pseudo_digest(&format!("network:{}", network.name), 25),
                "Addr": format!("{}/16", network.ipv4_address.unwrap_or_else(|| fake_ipv4(index + network_index))),
            })
        })
        .collect::<Vec<_>>()
}

fn iterate_networks(service: &ComposeService) -> Vec<ResolvedNetwork> {
    match service.networks.as_ref() {
        Some(ComposeNetworks::List(names)) => names
            .iter()
            .map(|name| ResolvedNetwork {
                name: name.clone(),
                aliases: vec![name.clone()],
                ipv4_address: None,
                ipv6_address: None,
            })
            .collect(),
        Some(ComposeNetworks::Map(map)) => map
            .iter()
            .map(|(name, config)| ResolvedNetwork {
                name: name.clone(),
                aliases: config.aliases.clone().unwrap_or_else(|| vec![name.clone()]),
                ipv4_address: config.ipv4_address.clone(),
                ipv6_address: config.ipv6_address.clone(),
            })
            .collect(),
        None => vec![ResolvedNetwork {
            name: "default".to_owned(),
            aliases: vec!["default".to_owned()],
            ipv4_address: None,
            ipv6_address: None,
        }],
    }
}

fn first_network_name(service: &ComposeService) -> Option<&str> {
    match service.networks.as_ref() {
        Some(ComposeNetworks::List(names)) => names.first().map(String::as_str),
        Some(ComposeNetworks::Map(map)) => map.keys().next().map(String::as_str),
        None => Some("default"),
    }
}

fn parse_port_mapping(port: &ComposePort) -> Option<ResolvedPort> {
    match port {
        ComposePort::Long(config) => Some(ResolvedPort {
            target: config.target,
            published: config.published,
            protocol: config.protocol.clone().unwrap_or_else(|| "tcp".to_owned()),
        }),
        ComposePort::Short(value) => {
            let (raw, protocol) = value
                .split_once('/')
                .map(|(raw, protocol)| (raw, protocol.to_owned()))
                .unwrap_or_else(|| (value.as_str(), "tcp".to_owned()));
            let parts = raw.split(':').collect::<Vec<_>>();
            let target = parts.last()?.parse::<u16>().ok()?;
            let published = if parts.len() >= 2 {
                parts.get(parts.len() - 2).and_then(|value| value.parse::<u16>().ok())
            } else {
                None
            };

            Some(ResolvedPort {
                target,
                published,
                protocol,
            })
        }
    }
}

fn expose_to_port(expose: &ComposeExpose) -> Option<u16> {
    match expose {
        ComposeExpose::Number(port) => Some(*port),
        ComposeExpose::String(value) => value
            .split('/')
            .next()
            .and_then(|value| value.parse::<u16>().ok()),
    }
}

fn parse_volume(volume: &ComposeVolume) -> Option<ResolvedMount> {
    match volume {
        ComposeVolume::Long(config) => {
            let destination = config.target.clone()?;
            let source = config.source.clone().unwrap_or_else(|| destination.clone());
            let mount_type = config.volume_type.clone().unwrap_or_else(|| {
                if source.starts_with('/') || source.starts_with('.') {
                    "bind".to_owned()
                } else {
                    "volume".to_owned()
                }
            });
            Some(ResolvedMount {
                mount_type: mount_type.clone(),
                name: (mount_type == "volume").then_some(source.clone()),
                source,
                destination,
                driver: (mount_type == "volume").then_some("local".to_owned()),
                mode: if config.read_only.unwrap_or(false) { "ro" } else { "rw" }.to_owned(),
                rw: !config.read_only.unwrap_or(false),
            })
        }
        ComposeVolume::Short(value) => {
            let parts = value.split(':').collect::<Vec<_>>();
            if parts.len() < 2 {
                return None;
            }
            let source = parts[0].to_owned();
            let destination = parts[1].to_owned();
            let read_only = parts.get(2).is_some_and(|mode| mode.contains("ro"));
            let mount_type = if source.starts_with('/') || source.starts_with('.') {
                "bind".to_owned()
            } else {
                "volume".to_owned()
            };
            Some(ResolvedMount {
                mount_type: mount_type.clone(),
                name: (mount_type == "volume").then_some(source.clone()),
                source,
                destination,
                driver: (mount_type == "volume").then_some("local".to_owned()),
                mode: parts.get(2).copied().unwrap_or("rw").to_owned(),
                rw: !read_only,
            })
        }
    }
}

fn fake_ipv4(index: usize) -> String {
    format!("10.255.{}.{}", (index / 200) + 1, (index % 200) + 10)
}

fn fake_ipv6(index: usize) -> String {
    format!("2001:db8::{:x}", index + 0x100)
}

fn pseudo_digest(input: &str, len: usize) -> String {
    let mut output = String::new();
    let mut seed = input.to_owned();

    while output.len() < len {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let value = hasher.finish();
        output.push_str(&format!("{value:016x}"));
        seed.push_str(&output);
    }

    output[..len].to_owned()
}

#[derive(Clone, Debug)]
struct ResolvedNetwork {
    name: String,
    aliases: Vec<String>,
    ipv4_address: Option<String>,
    ipv6_address: Option<String>,
}

#[derive(Clone, Debug)]
struct ResolvedPort {
    target: u16,
    published: Option<u16>,
    protocol: String,
}

#[derive(Clone, Debug)]
struct ResolvedMount {
    mount_type: String,
    name: Option<String>,
    source: String,
    destination: String,
    driver: Option<String>,
    mode: String,
    rw: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_COMPOSE: &str = r#"services:
  homepage:
    image: ghcr.io/gethomepage/homepage:latest
    container_name: homepage
    labels:
      kuma.home.http.name: Homepage
      kuma.home.http.url: https://homepage.example.com
      kuma.home.http.parent_name: core

  core-group:
    image: busybox
    labels:
      kuma.core.group.name: Core Services
"#;

    const TEST_CONFIG: &str = r#"[docker]
label_prefix = "kuma"
"#;

    fn build_engine() -> PlaygroundEngine {
        let config = parse_config(TEST_CONFIG, &ConfigFormat::Toml).unwrap();
        PlaygroundEngine::new(config).unwrap()
    }

    #[test]
    fn compose_entities_resolve_cross_target_group_references() {
        let engine = build_engine();
        let targets = parse_compose_targets(TEST_COMPOSE).unwrap();

        let entities = engine.collect_compose_entities(&targets).unwrap();

        let home = entities
            .iter()
            .find(|entity| entity.id == "home")
            .unwrap();

        match &home.entity {
            Entity::Monitor(monitor) => {
                assert_eq!(monitor.common().parent(), &Some(1));
            }
            entity => panic!("expected monitor, got {entity:?}"),
        }
    }

    #[test]
    fn snippets_can_reference_compose_defined_groups() {
        let engine = build_engine();
        let targets = parse_compose_targets(TEST_COMPOSE).unwrap();
        let compose_entities = engine.collect_compose_entities(&targets).unwrap();
        let target = targets
            .iter()
            .find(|target| target.id == "container:homepage")
            .unwrap();

        let snippet = r#"homepage_preview.http.name: Homepage Preview
homepage_preview.http.url: https://homepage-preview.example.com
homepage_preview.http.parent_name: core"#;

        let output = engine
            .render_snippet(target, snippet, &compose_entities)
            .unwrap();

        match &output.entities[0].entity {
            Entity::Monitor(monitor) => {
                assert_eq!(monitor.common().parent(), &Some(1));
            }
            entity => panic!("expected monitor, got {entity:?}"),
        }
    }
}
