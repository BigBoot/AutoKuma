use itertools::Itertools;
use serde_json::json;

use crate::{
    config::Config,
    error::{Error, Result},
    util::group_by_prefix,
};
use std::{collections::BTreeMap, sync::Arc};

pub struct AppState {
    pub config: Arc<Config>,
    defaults: BTreeMap<String, Vec<(String, String)>>,
}

impl AppState {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let defaults = config
            .default_settings
            .lines()
            .map(|line| {
                line.split_once(":")
                    .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                    .ok_or_else(|| {
                        Error::InvalidConfig("kuma.default_settings".to_owned(), line.to_owned())
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            config: config.clone(),
            defaults: group_by_prefix(defaults, "."),
        })
    }

    pub fn get_defaults(&self, monitor_type: impl AsRef<str>) -> Vec<(String, serde_json::Value)> {
        vec![
            self.defaults.get("*"),
            self.defaults.get(monitor_type.as_ref()),
        ]
        .into_iter()
        .flat_map(|defaults| {
            defaults
                .into_iter()
                .map(|entry| {
                    entry
                        .into_iter()
                        .map(|(key, value)| (key.to_owned(), json!(value)))
                })
                .collect_vec()
        })
        .flatten()
        .collect_vec()
    }
}
