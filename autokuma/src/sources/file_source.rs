use crate::{
    app_state::AppState,
    entity::{get_entity_from_value, Entity},
    error::{Error, Result},
    sources::source::Source,
};
use async_trait::async_trait;
use itertools::Itertools;
use kuma_client::util::ResultLogger;
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use walkdir::WalkDir;

async fn get_entities_from_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    state: Arc<AppState>,
    base_path: P1,
    file: P2,
) -> Result<Vec<(String, Entity)>> {
    let base_path = base_path.as_ref();
    let file = file.as_ref();
    let file_path = base_path.join(file);

    let value: Option<serde_json::Value> = if file.extension().is_some_and(|ext| ext == "json") {
        let content: String = tokio::fs::read_to_string(file_path)
            .await
            .map_err(|e| Error::IO(e.to_string()))?;

        Some(serde_json::from_str(&content).map_err(|e| Error::DeserializeError(e.to_string()))?)
    } else if file.extension().is_some_and(|ext| ext == "toml") {
        let content = tokio::fs::read_to_string(file_path)
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
        .map(|(id, value, context)| {
            get_entity_from_value(state.clone(), id.clone(), value, context).map(|e| (id, e))
        })
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

        let static_monitor_path = PathBuf::from(&static_monitor_path);
        if tokio::fs::metadata(&static_monitor_path)
            .await
            .is_ok_and(|md| md.is_dir())
        {
            let files = WalkDir::new(&static_monitor_path)
                .into_iter()
                .filter_map(|e| e.log_warn(std::module_path!(), |e| e.to_string()).ok())
                .filter(|e| e.file_type().is_file());

            for file in files {
                let file_path = file.path().strip_prefix(&static_monitor_path).unwrap();

                if let Ok(file_entities) =
                    get_entities_from_file(self.state.clone(), &static_monitor_path, file_path)
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
