use chrono::{self, DateTime};
use itertools::Itertools;
use kuma_client::Client;
use log::info;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sled::IVec;

use crate::{
    config::Config,
    error::{Error, Result},
    name::{EntitySelector, Name},
    util::group_by_prefix,
};
use core::str;
use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
};

fn decode_i32(value: &IVec) -> Result<i32> {
    value
        .as_ref()
        .try_into()
        .map(|v| i32::from_le_bytes(v))
        .map_err(|e| Error::InternalError(format!("Unable to read i32 from db: {}", e)))
}

fn encode_i32(id: i32) -> Result<IVec> {
    Ok(id.to_le_bytes().to_vec().into())
}

fn decode_string(value: &IVec) -> Result<String> {
    Ok(str::from_utf8(&value)
        .map_err(|e| Error::InternalError(format!("Unable to deserialize string from db: {}", e)))?
        .to_owned())
}

fn encode_string(id: String) -> Result<IVec> {
    Ok(id.as_bytes().to_vec().into())
}

pub(crate) fn encode_value<V>(value: V) -> Result<IVec>
where
    V: serde::Serialize,
{
    Ok(postcard::to_allocvec(&value)
        .map_err(|e| Error::InternalError(format!("Unable to encode db entry: {}", e)))?
        .into())
}

pub(crate) fn decode_value<V>(value: IVec) -> Result<V>
where
    V: serde::de::DeserializeOwned,
{
    postcard::from_bytes(&value)
        .map_err(|e| Error::InternalError(format!("Unable to decode db entry: {}", e)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEntry {
    pub delete_at: chrono::DateTime<chrono::Utc>,
    pub entity: EntitySelector,
}

pub(crate) struct AppDBTables {
    pub monitors: DBTable<String, i32>,
    pub to_delete: DBTable<IVec, DeleteEntry>,
    pub notifications: DBTable<String, i32>,
    pub docker_hosts: DBTable<String, i32>,
    pub tags: DBTable<String, i32>,
    pub status_pages: DBTable<String, String>,
}

pub struct AppDB {
    db: sled::Db,
    tables: AppDBTables,
}

trait IDTable<T> {
    fn read_id(&self, value: &IVec) -> Result<T>;
    fn store_id(&self, id: T) -> Result<IVec>;
    fn tree(&self) -> &sled::Tree;
}

pub(crate) trait ValueTable<V> {
    fn encode_value(value: V) -> Result<IVec>;
    fn decode_value(value: IVec) -> Result<V>;
}

#[allow(dead_code)]
pub(crate) trait KeyTable<K> {
    fn encode_key(key: K) -> Result<IVec>;
    fn decode_key(key: IVec) -> Result<K>;
}

#[allow(dead_code)]
trait KeyValueTable<K, V> {
    fn read_value(&self, key: K) -> Result<Option<V>>;
    fn store_value(&self, key: K, value: V) -> Result<()>;
}

pub(crate) struct DBTable<K, V> {
    pub(crate) tree: sled::Tree,
    _k: std::marker::PhantomData<K>,
    _v: std::marker::PhantomData<V>,
}

impl<K, V> DBTable<K, V> {
    fn new(db: &sled::Db, name: &str) -> Result<Self> {
        Ok(DBTable {
            tree: db.open_tree(name)?,
            _k: PhantomData,
            _v: PhantomData,
        })
    }
}

impl IDTable<i32> for DBTable<String, i32> {
    fn read_id(&self, value: &IVec) -> Result<i32> {
        decode_i32(value)
    }

    fn store_id(&self, id: i32) -> Result<IVec> {
        encode_i32(id)
    }

    fn tree(&self) -> &sled::Tree {
        &self.tree
    }
}

impl IDTable<String> for DBTable<String, String> {
    fn read_id(&self, value: &IVec) -> Result<String> {
        decode_string(value)
    }

    fn store_id(&self, id: String) -> Result<IVec> {
        encode_string(id)
    }

    fn tree(&self) -> &sled::Tree {
        &self.tree
    }
}

impl<V> KeyTable<String> for DBTable<String, V> {
    fn encode_key(key: String) -> Result<IVec> {
        encode_string(key)
    }

    fn decode_key(key: IVec) -> Result<String> {
        decode_string(&key)
    }
}

impl<V> KeyTable<i32> for DBTable<i32, V> {
    fn encode_key(key: i32) -> Result<IVec> {
        encode_i32(key)
    }

    fn decode_key(key: IVec) -> Result<i32> {
        decode_i32(&key)
    }
}

impl<V> KeyTable<IVec> for DBTable<IVec, V> {
    fn encode_key(key: IVec) -> Result<IVec> {
        Ok(key)
    }

    fn decode_key(key: IVec) -> Result<IVec> {
        Ok(key)
    }
}

impl<'de, K, V> ValueTable<V> for DBTable<K, V>
where
    V: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode_value(value: V) -> Result<IVec> {
        encode_value(value)
    }

    fn decode_value(value: IVec) -> Result<V> {
        decode_value(value)
    }
}

impl<K, V> KeyValueTable<K, V> for DBTable<K, V>
where
    Self: KeyTable<K> + ValueTable<V>,
{
    fn read_value(&self, key: K) -> Result<Option<V>> {
        Ok(self
            .tree
            .get(Self::encode_key(key)?)?
            .map(Self::decode_value)
            .transpose()?)
    }

    fn store_value(&self, key: K, value: V) -> Result<()> {
        self.tree
            .insert(Self::encode_key(key)?, Self::encode_value(value)?)?;

        Ok(())
    }
}

impl<K, V> DBTable<K, V>
where
    K: 'static,
    V: 'static + serde::Serialize + serde::de::DeserializeOwned,
    Self: KeyTable<K> + ValueTable<V>,
{
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (K, V)> + 'a> {
        Box::new(self.tree.iter().flat_map(|entry| match entry {
            Ok((key, value)) => match (Self::decode_key(key), Self::decode_value(value)) {
                (Ok(key), Ok(value)) => Some((key, value)),
                _ => None,
            },
            _ => None,
        }))
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
            tables: AppDBTables {
                monitors: DBTable::new(&db, "monitors")?,
                to_delete: DBTable::new(&db, "to_delete")?,
                notifications: DBTable::new(&db, "notifications")?,
                docker_hosts: DBTable::new(&db, "docker_hosts")?,
                tags: DBTable::new(&db, "tags")?,
                status_pages: DBTable::new(&db, "status_pages")?,
            },
            db,
        })
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
            Name::Monitor(name) => Self::get_value(&self.tables.monitors, &name)?,
            Name::Notification(name) => Self::get_value(&self.tables.notifications, &name)?,
            Name::DockerHost(name) => Self::get_value(&self.tables.docker_hosts, &name)?,
            Name::Tag(name) => Self::get_value(&self.tables.tags, &name)?,
            Name::StatusPage(name) => Self::get_value(&self.tables.status_pages, &name)?,
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
                .tables
                .monitors
                .tree()
                .insert(name, self.tables.monitors.store_id(id)?)?,
            (Name::Notification(name), DatabaseId::I32(id)) => self
                .tables
                .notifications
                .tree()
                .insert(name, self.tables.notifications.store_id(id)?)?,
            (Name::DockerHost(name), DatabaseId::I32(id)) => self
                .tables
                .docker_hosts
                .tree()
                .insert(name, self.tables.docker_hosts.store_id(id)?)?,
            (Name::Tag(name), DatabaseId::I32(id)) => {
                self.tables.tags.tree().insert(name, self.tables.tags.store_id(id)?)?
            }
            (Name::StatusPage(name), DatabaseId::String(id)) => self
                .tables
                .status_pages
                .tree()
                .insert(name, self.tables.status_pages.store_id(id)?)?,
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
                Ok((decode_string(&name)?, table.read_id(&value)?))
            })
            .collect::<Result<Vec<_>>>()?)
    }

    pub fn remove_id(&self, name: Name) -> Result<()> {
        let (tree, key) = match &name {
            Name::Monitor(name) => (&self.tables.monitors.tree(), name),
            Name::Notification(name) => (&self.tables.notifications.tree(), name),
            Name::DockerHost(name) => (&self.tables.docker_hosts.tree(), name),
            Name::Tag(name) => (&self.tables.tags.tree(), name),
            Name::StatusPage(name) => (&self.tables.status_pages.tree(), name),
        };

        tree.remove(key)?;

        self.tables.to_delete.tree.remove(encode_value(name)?)?;

        Ok(())
    }

    fn clean_table<T: Eq + Hash>(&self, table: &impl IDTable<T>, ids: &HashSet<T>) -> Result<()> {
        let to_delete = table
            .tree()
            .iter()
            .filter_map(|e| e.ok())
            .filter(|(_, value)| !ids.contains(&table.read_id(value).unwrap()));

        let mut batch = sled::Batch::default();

        for (key, value) in to_delete {
            log::debug!("Removing {}", String::from_utf8_lossy(&value));
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
        self.clean_table(&self.tables.monitors, monitors)?;
        self.clean_table(&self.tables.notifications, notifications)?;
        self.clean_table(&self.tables.docker_hosts, docker_hosts)?;
        self.clean_table(&self.tables.tags, tags)?;
        self.clean_table(&self.tables.status_pages, status_pages)?;

        Ok(())
    }

    pub fn get_monitors(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.tables.monitors)
    }

    pub fn request_to_delete(
        &self,
        entity: EntitySelector,
        delete_at: DateTime<chrono::Utc>,
    ) -> Result<()> {
        _ = self.tables.to_delete.tree.compare_and_swap(
            encode_value(entity.clone())?,
            None as Option<&[u8]>,
            Some(DBTable::<String, DeleteEntry>::encode_value(DeleteEntry {
                delete_at,
                entity,
            })?),
        );

        Ok(())
    }

    pub fn get_entities_to_delete(&self) -> Result<Vec<EntitySelector>> {
        let now = chrono::Utc::now();
        let to_delete = self
            .tables
            .to_delete
            .iter()
            .filter(|(_, entry)| entry.delete_at < now)
            .collect::<Vec<_>>();

        let mut batch = sled::Batch::default();
        for (key, _) in to_delete.iter() {
            batch.remove(key);
        }
        self.tables.to_delete.tree.apply_batch(batch)?;

        Ok(to_delete
            .into_iter()
            .map(|(_, entry)| entry.entity)
            .collect())
    }

    pub fn get_notifications(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.tables.notifications)
    }

    pub fn get_docker_hosts(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.tables.docker_hosts)
    }

    pub fn get_tags(&self) -> Result<Vec<(String, i32)>> {
        Self::get_entries(&self.tables.tags)
    }

    pub fn pending_deletion_count(&self) -> usize {
        self.tables.to_delete.tree.len()
    }

    pub fn get_status_pages(&self) -> Result<Vec<(String, String)>> {
        Self::get_entries(&self.tables.status_pages)
    }

    pub fn get_version(&self) -> Result<i32> {
        Ok(self
            .db
            .get("version")?
            .map(|value| decode_i32(&value))
            .transpose()?
            .unwrap_or(0))
    }

    pub fn set_version(&self, version: i32) -> Result<()> {
        self.db.insert("version", &version.to_le_bytes())?;
        Ok(())
    }

    pub async fn migrate(&self, state: &AppState, kuma: &Client) -> Result<()> {
        use crate::migrations::{CURRENT_VERSION, MIGRATIONS};
        loop {
            let version = self.get_version()?;

            if version > *CURRENT_VERSION {
                log::error!(
                    "Database version {} is higher than the current version ({}), refusing to continue.",
                    version,
                    *CURRENT_VERSION
                );
                return Ok(());
            }

            if version < *CURRENT_VERSION {
                info!("Migrating database to version {}", version + 1);
                let migration = MIGRATIONS[version as usize];
                migration(&self.tables, state, kuma).await?;
                self.set_version(version + 1)?;
                continue;
            }

            break;
        }

        Ok(())
    }
}

pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<AppDB>,
    defaults: BTreeMap<String, Vec<(String, String)>>,
    pub compiled_exclusion_patterns: Vec<Regex>,
}

impl AppState {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let defaults = config
            .default_settings
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.split_once(":")
                    .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                    .ok_or_else(|| {
                        Error::InvalidConfig("default_settings".to_owned(), line.to_owned())
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

        // Compile exclusion patterns at startup
        let compiled_exclusion_patterns = config
            .docker
            .exclude_container_patterns
            .as_ref()
            .map(|patterns| {
                patterns
                    .iter()
                    .filter_map(|pattern| match Regex::new(pattern) {
                        Ok(regex) => Some(regex),
                        Err(e) => {
                            log::error!(
                                "Invalid container exclusion pattern '{}': {}. Pattern will be ignored.",
                                pattern,
                                e
                            );
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new);

        Ok(Self {
            db: Arc::new(AppDB::new(&data_path)?),
            config,
            defaults: group_by_prefix(defaults, "."),
            compiled_exclusion_patterns,
        })
    }

    pub fn get_defaults(&self, monitor_type: impl AsRef<str>) -> Vec<(String, serde_json::Value)> {
        vec![
            self.defaults.get(monitor_type.as_ref()),
            self.defaults.get("*"),
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
