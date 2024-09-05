use crate::{
    app_state::AppState,
    entity::{get_entity_from_settings, Entity},
    error::{Error, Result},
    util::FlattenValue,
};

pub async fn get_entity_from_file(
    state: &AppState,
    file: impl AsRef<str>,
) -> Result<(String, Entity)> {
    let file = file.as_ref();
    let id = std::path::Path::new(file)
        .file_stem()
        .and_then(|os| os.to_str().map(|str| str.to_owned()))
        .ok_or_else(|| Error::IO(format!("Unable to determine file: '{}'", file)))?;

    let value: Option<serde_json::Value> = if file.ends_with(".json") {
        let content: String = tokio::fs::read_to_string(file)
            .await
            .map_err(|e| Error::IO(e.to_string()))?;

        Some(serde_json::from_str(&content).map_err(|e| Error::DeserializeError(e.to_string()))?)
    } else if file.ends_with(".toml") {
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
                file
            ))
        })
        .and_then(|v| v.flatten())?;

    let entity_type = values
        .iter()
        .find(|(key, _)| key == "type")
        .and_then(|(_, value)| value.as_str().map(|s| s.to_owned()))
        .ok_or_else(|| {
            Error::DeserializeError(format!("Static monitor {} is missing `type`", file))
        })?;

    let context = tera::Context::new();
    let monitor = get_entity_from_settings(state, &id, &entity_type, values, &context)?;

    Ok((id, monitor))
}
