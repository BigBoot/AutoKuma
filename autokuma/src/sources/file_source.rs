use std::path::PathBuf;

use async_trait::async_trait;
use itertools::Itertools;
use walkdir::WalkDir;

use crate::{
    app_state::AppState,
    entity::{get_entity_from_settings, Entity},
    error::{Error, Result},
    sources::source::Source,
    util::FlattenValue,
};
use kuma_client::util::ResultLogger;

pub async fn get_entity_from_file(state: &AppState, file: &PathBuf) -> Result<(String, Entity)> {
    let id = file
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

    let values = value
        .ok_or_else(|| {
            Error::DeserializeError(format!(
                "Unsupported static monitor file type: {}, supported: .json, .toml",
                file.display()
            ))
        })
        .and_then(|v| v.flatten())?;

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

    let context = tera::Context::new();
    let entity = get_entity_from_settings(state, &id, &entity_type, values, &context)?;

    Ok((id, entity))
}

pub struct FileSource {}

#[async_trait]
impl Source for FileSource {
    async fn get_entities(&mut self, state: &AppState) -> Result<Vec<(String, Entity)>> {
        let mut entities = vec![];

        let static_monitor_path = state
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
                if let Ok(entity) = get_entity_from_file(&state, &file.path().to_path_buf())
                    .await
                    .log_warn(std::module_path!(), |e| {
                        format!("[{}] {}", file.path().display(), e)
                    })
                {
                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }
}
