use itertools::Itertools;
use serde_json::json;
use sled::{IVec, Tree};

use crate::{
    config::Config,
    error::{Error, Result},
    name::Name,
    util::group_by_prefix,
};
use core::str;
use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

pub struct AppDB {
    db: sled::Db,
    monitors: sled::Tree,
    notifications: sled::Tree,
    docker_hosts: sled::Tree,
    tags: sled::Tree,
}

impl AppDB {
    fn read_i32(value: &IVec) -> Result<i32> {
        value
            .as_ref()
            .try_into()
            .map(|v| i32::from_le_bytes(v))
            .map_err(|_| Error::DeserializeError("Unable to deserialize i32 from db".to_owned()))
    }

    fn read_string(value: &IVec) -> Result<String> {
        Ok(str::from_utf8(&value)
            .map_err(|e| {
                Error::DeserializeError(format!("Unable to deserialize string from db: {}", e))
            })?
            .to_owned())
    }

    fn get_i32(tree: &Tree, name: String) -> Result<Option<i32>> {
        tree.get(name)?
            .map(|value| Self::read_i32(&value))
            .transpose()
    }

    fn store_i32(tree: &Tree, name: String, id: i32) -> Result<()> {
        tree.insert(name, &id.to_le_bytes())?;
        Ok(())
    }

    fn get_entries(tree: &Tree) -> Result<Vec<(String, i32)>> {
        Ok(tree
            .iter()
            .map(|entry| {
                let (name, value) = entry?;
                Ok((Self::read_string(&name)?, Self::read_i32(&value)?))
            })
            .collect::<Result<Vec<_>>>()?)
    }

    pub fn get_id(&self, name: Name) -> Result<Option<i32>> {
        let (tree, name) = match name {
            Name::Monitor(name) => (&self.monitors, name),
            Name::Notification(name) => (&self.notifications, name),
            Name::DockerHost(name) => (&self.docker_hosts, name),
            Name::Tag(name) => (&self.tags, name),
        };

        Self::get_i32(tree, name)
    }

    pub fn store_id(&self, name: Name, id: i32) -> Result<()> {
        let (tree, name) = match name {
            Name::Monitor(name) => (&self.monitors, name),
            Name::Notification(name) => (&self.notifications, name),
            Name::DockerHost(name) => (&self.docker_hosts, name),
            Name::Tag(name) => (&self.tags, name),
        };

        Self::store_i32(tree, name, id)
    }

    pub fn remove_id(&self, name: Name) -> Result<()> {
        let (tree, name) = match name {
            Name::Monitor(name) => (&self.monitors, name),
            Name::Notification(name) => (&self.notifications, name),
            Name::DockerHost(name) => (&self.docker_hosts, name),
            Name::Tag(name) => (&self.tags, name),
        };

        tree.remove(name)?;
        Ok(())
    }

    pub fn clean(
        &self,
        monitors: &HashSet<i32>,
        notifications: &HashSet<i32>,
        docker_hosts: &HashSet<i32>,
        tags: &HashSet<i32>,
    ) -> Result<()> {
        for (tree, ids) in &[
            (&self.monitors, monitors),
            (&self.notifications, notifications),
            (&self.docker_hosts, docker_hosts),
            (&self.tags, tags),
        ] {
            let to_delete = tree
                .iter()
                .filter_map(|e| e.ok())
                .filter(|(_, value)| !ids.contains(&Self::read_i32(value).unwrap_or(-1)))
                .map(|(key, _)| key);

            let mut batch = sled::Batch::default();

            for key in to_delete {
                batch.remove(key);
            }

            tree.apply_batch(batch)?;
        }

        Ok(())
    }

    pub fn get_monitors(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.monitors)
    }

    pub fn get_notifications(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.notifications)
    }

    pub fn get_docker_hosts(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.docker_hosts)
    }

    pub fn get_tags(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.tags)
    }

    pub fn get_version(&self) -> Result<i32> {
        Ok(self
            .db
            .get("version")?
            .map(|value| Self::read_i32(&value))
            .transpose()?
            .unwrap_or(0))
    }

    pub fn set_version(&self, version: i32) -> Result<()> {
        self.db.insert("version", &version.to_le_bytes())?;
        Ok(())
    }
}

pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<AppDB>,
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

        let data_path =
            config
                .data_path
                .clone()
                .unwrap_or_else(|| match std::env::var_os("AUTOKUMA_DOCKER") {
                    Some(_) => "/data".to_owned(),
                    None => dirs::config_local_dir()
                        .map(|dir| {
                            dir.join("autokuma")
                                .join("config")
                                .to_string_lossy()
                                .to_string()
                        })
                        .unwrap_or_else(|| "./".to_owned()),
                });
        let db = sled::open(format!("{}/autokuma.db", data_path))?;
        let app_db: Arc<AppDB> = Arc::new(AppDB {
            monitors: db.open_tree("monitors")?,
            notifications: db.open_tree("notifications")?,
            docker_hosts: db.open_tree("docker_hosts")?,
            tags: db.open_tree("tags")?,
            db: db,
        });

        Ok(Self {
            db: app_db,
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
