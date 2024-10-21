use crate::{
    app_state::AppState,
    entity::{get_entity_from_settings, Entity},
    error::{Error, Result},
    sources::source::Source,
    util::FlattenValue,
};
use async_trait::async_trait;
use itertools::Itertools;
use kuma_client::util::ResultLogger;
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
use walkdir::WalkDir;

fn get_entity_from_value(
    state: Arc<AppState>,
    id: String,
    file: &PathBuf,
    value: serde_json::Value,
    context: tera::Context,
) -> Result<(String, Entity)> {
    let values = value.flatten()?;

    let entity_type = values
        .iter()
        .find(|(key, _)| key == "type")
        .and_then(|(_, value)| value.as_str().map(|s| s.to_owned()))
        .ok_or_else(|| {
            Error::DeserializeError(format!(
                "Static monitor {} is missing `type`",
                file.display()
            ))
        })?;

    let entity = get_entity_from_settings(state, &id, &entity_type, values, &context)?;

    Ok((id, entity))
}

pub async fn get_entities_from_file(
    state: Arc<AppState>,
    file: &PathBuf,
) -> Result<Vec<(String, Entity)>> {
    let value: Option<serde_json::Value> = if file.extension().is_some_and(|ext| ext == "json") {
        let content: String = tokio::fs::read_to_string(file)
            .await
            .map_err(|e| Error::IO(e.to_string()))?;

        Some(serde_json::from_str(&content).map_err(|e| Error::DeserializeError(e.to_string()))?)
    } else if file.extension().is_some_and(|ext| ext == "toml") {
        let content = tokio::fs::read_to_string(file)
            .await
            .map_err(|e| Error::IO(e.to_string()))?;

        Some(toml::from_str(&content).map_err(|e| Error::DeserializeError(e.to_string()))?)
    } else {
        None
    };

    let file_id: String = file
        .ancestors()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .filter_map(|p| match p == file {
            true => p.file_stem().map(|s| s.to_string_lossy()),
            false => p.file_name().map(|s| s.to_string_lossy()),
        })
        .skip(1)
        .join("/");

    let value = value.ok_or_else(|| {
        Error::DeserializeError(format!(
            "Unsupported static monitor file type: {}, supported: .json, .toml",
            file.display()
        ))
    })?;

    let values = match value {
        serde_json::Value::Array(entities) => entities
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                (
                    format!("{}[{}]", file_id, i),
                    value,
                    tera::Context::from_value(json!({
                        "file_index": i,
                    }))
                    .unwrap(),
                )
            })
            .collect(),
        _ => vec![(file_id, value, tera::Context::new())],
    };

    let entities = values
        .into_iter()
        .map(|(id, value, context)| get_entity_from_value(state.clone(), id, &file, value, context))
        .into_iter()
        .filter_map(|r| {
            r.log_warn(std::module_path!(), |e| {
                format!("[{}] {}", file.display(), e)
            })
            .ok()
        })
        .collect();

    return Ok(entities);
}

pub struct FileSource {
    state: Arc<AppState>,
}

#[async_trait]
impl Source for FileSource {
    fn name(&self) -> &'static str {
        "File"
    }

    async fn init(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    async fn get_entities(&mut self) -> Result<Vec<(String, Entity)>> {
        let mut entities = vec![];

        let static_monitor_path = self
            .state
            .config
            .static_monitors
            .clone()
            .unwrap_or_else(|| {
                dirs::config_local_dir()
                    .map(|dir| {
                        dir.join("autokuma")
                            .join("static-monitors")
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_default()
            })
            .to_owned();

        if tokio::fs::metadata(&static_monitor_path)
            .await
            .is_ok_and(|md| md.is_dir())
        {
            let files = WalkDir::new(&static_monitor_path)
                .into_iter()
                .filter_map(|e| e.log_warn(std::module_path!(), |e| e.to_string()).ok())
                .filter(|e| e.file_type().is_file());

            for file in files {
                if let Ok(file_entities) =
                    get_entities_from_file(self.state.clone(), &file.path().to_path_buf())
                        .await
                        .log_warn(std::module_path!(), |e| {
                            format!("[{}] {}", file.path().display(), e)
                        })
                {
                    entities.extend(file_entities);
                }
            }
        }

        Ok(entities)
    }
}

impl FileSource {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}
