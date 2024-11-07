use crate::{
    config::Config,
    error::{Error, Result},
};
use serde_json::json;
use std::{collections::BTreeMap, error::Error as _, sync::Arc};
use tera::Tera;

pub fn group_by_prefix<A, B, I>(v: I, delimiter: &str) -> BTreeMap<String, Vec<(String, String)>>
where
    A: AsRef<str>,
    B: AsRef<str>,
    I: IntoIterator<Item = (A, B)>,
{
    v.into_iter()
        .fold(BTreeMap::new(), |mut groups, (key, value)| {
            if let Some((prefix, key)) = key.as_ref().split_once(delimiter) {
                groups
                    .entry(prefix.to_owned())
                    .or_default()
                    .push((key.to_owned(), value.as_ref().to_owned()));
            }
            groups
        })
}

pub trait ResultOrDie<T> {
    fn unwrap_or_die(self, exit_code: i32) -> T;
}

impl<T, E> ResultOrDie<T> for std::result::Result<T, E> {
    fn unwrap_or_die(self, exit_code: i32) -> T {
        match self {
            Ok(t) => t,
            Err(_) => std::process::exit(exit_code),
        }
    }
}

pub trait FlattenValue {
    fn flatten(&self) -> Result<Vec<(String, serde_json::Value)>>;
}

impl FlattenValue for serde_json::Value {
    fn flatten(&self) -> Result<Vec<(String, serde_json::Value)>> {
        let mut map = serde_json::Map::new();
        insert_object(
            &mut map,
            None,
            self.as_object()
                .ok_or_else(|| Error::DeserializeError("Not an object".to_string()))?,
        )?;
        Ok(map.into_iter().collect())
    }
}

fn insert_object(
    base_json: &mut serde_json::Map<String, serde_json::Value>,
    base_key: Option<&str>,
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<()> {
    for (key, value) in object {
        let new_key = base_key.map_or_else(|| key.clone(), |base_key| format!("{base_key}.{key}"));

        if let Some(object) = value.as_object() {
            insert_object(base_json, Some(&new_key), object)?;
        } else if let Some(array) = value.as_array() {
            base_json.insert(
                new_key.to_string(),
                json!(serde_json::to_string(&array)
                    .map_err(|e| Error::DeserializeError(e.to_string()))?),
            );
        } else {
            base_json.insert(new_key.to_string(), json!(value));
        }
    }

    Ok(())
}

struct GetEnvFunction {
    config: Arc<Config>,
}

impl tera::Function for GetEnvFunction {
    fn call(
        &self,
        args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let name = match args.get("name") {
            Some(val) => match tera::from_value::<String>(val.clone()) {
                Ok(v) => v,
                Err(_) => {
                    return Err(tera::Error::msg(format!(
                        "Function `get_env` received name={} but `name` can only be a string",
                        val
                    )));
                }
            },
            None => {
                return Err(tera::Error::msg(
                    "Function `get_env` didn't receive a `name` argument",
                ))
            }
        };

        if !self.config.insecure_env_access && !name.starts_with("AUTOKUMA__ENV__") {
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
    }
}

pub fn fill_templates(
    config: Arc<Config>,
    template: impl Into<String>,
    template_values: &tera::Context,
) -> Result<String> {
    let template = template.into();
    let mut tera = Tera::default();
    let get_env = GetEnvFunction {
        config: config.clone(),
    };

    tera.register_function("get_env", get_env);

    tera.add_raw_template(&template, &template)
        .and_then(|_| tera.render(&template, template_values))
        .map_err(|e| {
            Error::LabelParseError(format!(
                "{}\nContext: {:?}",
                e.source().unwrap(),
                &template_values.get("container")
            ))
        })
}
