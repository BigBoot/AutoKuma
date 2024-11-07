use itertools::Itertools;
use serde_json::json;
use sled::IVec;

use crate::{
    config::Config,
    error::{Error, Result},
    name::Name,
    util::group_by_prefix,
};
use core::str;
use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
};

fn read_i32(value: &IVec) -> Result<i32> {
    value
        .as_ref()
        .try_into()
        .map(|v| i32::from_le_bytes(v))
        .map_err(|e| Error::InternalError(format!("Unable to read i32 from db: {}", e)))
}

fn store_i32(id: i32) -> Result<IVec> {
    Ok(id.to_le_bytes().to_vec().into())
}

fn read_string(value: &IVec) -> Result<String> {
    Ok(str::from_utf8(&value)
        .map_err(|e| Error::InternalError(format!("Unable to deserialize string from db: {}", e)))?
        .to_owned())
}

fn store_string(id: String) -> Result<IVec> {
    Ok(id.as_bytes().to_vec().into())
}

pub struct AppDB {
    db: sled::Db,
    monitors: DBTable<i32>,
    notifications: DBTable<i32>,
    docker_hosts: DBTable<i32>,
    tags: DBTable<i32>,
    status_pages: DBTable<String>,
}

trait IDTable<T> {
    fn read_id(&self, value: &IVec) -> Result<T>;
    fn store_id(&self, id: T) -> Result<IVec>;
    fn tree(&self) -> &sled::Tree;
}

struct DBTable<T> {
    tree: sled::Tree,
    _t: std::marker::PhantomData<T>,
}

impl<T> DBTable<T> {
    fn new(db: &sled::Db, name: &str) -> Result<Self> {
        Ok(DBTable {
            tree: db.open_tree(name)?,
            _t: PhantomData,
        })
    }
}

impl IDTable<i32> for DBTable<i32> {
    fn read_id(&self, value: &IVec) -> Result<i32> {
        read_i32(value)
    }

    fn store_id(&self, id: i32) -> Result<IVec> {
        store_i32(id)
    }

    fn tree(&self) -> &sled::Tree {
        &self.tree
    }
}

impl IDTable<String> for DBTable<String> {
    fn read_id(&self, value: &IVec) -> Result<String> {
        read_string(value)
    }

    fn store_id(&self, id: String) -> Result<IVec> {
        store_string(id)
    }

    fn tree(&self) -> &sled::Tree {
        &self.tree
    }
}

pub enum DatabaseId {
    String(String),
    I32(i32),
}

impl From<String> for DatabaseId {
    fn from(value: String) -> Self {
        DatabaseId::String(value)
    }
}

impl From<i32> for DatabaseId {
    fn from(value: i32) -> Self {
        DatabaseId::I32(value)
    }
}

impl TryFrom<DatabaseId> for i32 {
    type Error = Error;

    fn try_from(value: DatabaseId) -> Result<i32> {
        match value {
            DatabaseId::I32(id) => Ok(id),
            DatabaseId::String(_) => {
                Err(Error::InternalError("DatabaseId is not an i32".to_owned()))
            }
        }
    }
}

impl TryFrom<DatabaseId> for String {
    type Error = Error;

    fn try_from(value: DatabaseId) -> Result<String> {
        match value {
            DatabaseId::String(id) => Ok(id),
            DatabaseId::I32(_) => Err(Error::InternalError(
                "DatabaseId is not a string".to_owned(),
            )),
        }
    }
}

impl AppDB {
    pub fn new(data_path: &str) -> Result<Self> {
        let db = sled::open(format!("{}/autokuma.db", data_path))?;
        Ok(AppDB {
            monitors: DBTable::new(&db, "monitors")?,
            notifications: DBTable::new(&db, "notifications")?,
            docker_hosts: DBTable::new(&db, "docker_hosts")?,
            tags: DBTable::new(&db, "tags")?,
            status_pages: DBTable::new(&db, "status_pages")?,
            db: db,
        })
    }

    fn read_string(value: &IVec) -> Result<String> {
        Ok(str::from_utf8(&value)
            .map_err(|e| {
                Error::DeserializeError(format!("Unable to deserialize string from db: {}", e))
            })?
            .to_owned())
    }

    fn get_value<T>(table: &impl IDTable<T>, name: &str) -> Result<Option<DatabaseId>>
    where
        DatabaseId: From<T>,
    {
        let value = table
            .tree()
            .get(name)?
            .map(|value| table.read_id(&value))
            .transpose()?
            .map(|value| DatabaseId::from(value));

        Ok(value)
    }

    pub fn get_id<T: TryFrom<DatabaseId>>(&self, name: Name) -> Result<Option<T>> {
        let id = match &name {
            Name::Monitor(name) => Self::get_value(&self.monitors, &name)?,
            Name::Notification(name) => Self::get_value(&self.notifications, &name)?,
            Name::DockerHost(name) => Self::get_value(&self.docker_hosts, &name)?,
            Name::Tag(name) => Self::get_value(&self.tags, &name)?,
            Name::StatusPage(name) => Self::get_value(&self.status_pages, &name)?,
        };

        id.map(|id| T::try_from(id)).transpose().map_err(|_| {
            Error::InternalError(format!(
                "Invalid key type {} for name {}",
                std::any::type_name::<T>(),
                name.type_name(),
            ))
        })
    }

    pub fn store_id<T: Into<DatabaseId>>(&self, name: Name, id: T) -> Result<()> {
        let id = id.into();
        match (&name, id) {
            (Name::Monitor(name), DatabaseId::I32(id)) => self
                .monitors
                .tree()
                .insert(name, self.monitors.store_id(id)?)?,
            (Name::Notification(name), DatabaseId::I32(id)) => self
                .notifications
                .tree()
                .insert(name, self.notifications.store_id(id)?)?,
            (Name::DockerHost(name), DatabaseId::I32(id)) => self
                .docker_hosts
                .tree()
                .insert(name, self.docker_hosts.store_id(id)?)?,
            (Name::Tag(name), DatabaseId::I32(id)) => {
                self.tags.tree().insert(name, self.tags.store_id(id)?)?
            }
            (Name::StatusPage(name), DatabaseId::String(id)) => self
                .status_pages
                .tree()
                .insert(name, self.status_pages.store_id(id)?)?,
            _ => Err(Error::InternalError(format!(
                "Invalid key type {} for Name {}",
                std::any::type_name::<T>(),
                name.type_name()
            )))?,
        };

        Ok(())
    }

    fn get_entries<T>(table: &impl IDTable<T>) -> Result<Vec<(String, T)>> {
        Ok(table
            .tree()
            .iter()
            .map(|entry| {
                let (name, value) = entry?;
                Ok((Self::read_string(&name)?, table.read_id(&value)?))
            })
            .collect::<Result<Vec<_>>>()?)
    }

    pub fn remove_id(&self, name: Name) -> Result<()> {
        let (tree, name) = match name {
            Name::Monitor(name) => (&self.monitors.tree(), name),
            Name::Notification(name) => (&self.notifications.tree(), name),
            Name::DockerHost(name) => (&self.docker_hosts.tree(), name),
            Name::Tag(name) => (&self.tags.tree(), name),
            Name::StatusPage(name) => (&self.status_pages.tree(), name),
        };

        tree.remove(name)?;
        Ok(())
    }

    fn clean_table<T: Eq + Hash>(table: &impl IDTable<T>, ids: &HashSet<T>) -> Result<()> {
        let to_delete = table
            .tree()
            .iter()
            .filter_map(|e| e.ok())
            .filter(|(_, value)| !ids.contains(&table.read_id(value).unwrap()));

        let mut batch = sled::Batch::default();

        for (key, value) in to_delete {
            println!("Removing {}", String::from_utf8_lossy(&value));
            batch.remove(key);
        }

        table.tree().apply_batch(batch)?;
        Ok(())
    }

    pub fn clean(
        &self,
        monitors: &HashSet<i32>,
        notifications: &HashSet<i32>,
        docker_hosts: &HashSet<i32>,
        tags: &HashSet<i32>,
        status_pages: &HashSet<String>,
    ) -> Result<()> {
        Self::clean_table(&self.monitors, monitors)?;
        Self::clean_table(&self.notifications, notifications)?;
        Self::clean_table(&self.docker_hosts, docker_hosts)?;
        Self::clean_table(&self.tags, tags)?;
        Self::clean_table(&self.status_pages, status_pages)?;

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

    pub fn get_status_pages(&self) -> Result<Vec<(String, String)>> {
        Self::get_entries(&self.status_pages)
    }

    pub fn get_version(&self) -> Result<i32> {
        Ok(self
            .db
            .get("version")?
            .map(|value| read_i32(&value))
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

        Ok(Self {
            db: Arc::new(AppDB::new(&data_path)?),
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
