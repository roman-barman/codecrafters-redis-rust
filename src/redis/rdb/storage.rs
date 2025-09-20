use crate::redis::rdb::read_database::read_first_database;
use crate::redis::rdb::ttl::Ttl;
use crate::redis::rdb::write_database::write_database;
use chrono::Utc;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct RedisStorage {
    storage: HashMap<String, (String, Ttl)>,
}

impl RedisStorage {
    pub fn restore_database(&mut self, path: &PathBuf) -> Result<(), RedisStorageError> {
        let mut file = File::open(path).map_err(|_| RedisStorageError {
            msg: format!("error opening file: {}", path.to_str().unwrap()),
        })?;
        let db = read_first_database(&mut file).map_err(|e| RedisStorageError {
            msg: format!("error restore database: {}", e),
        })?;
        if let Some(db) = db {
            self.storage = db;
        }
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<&str> {
        let should_remove = match self.storage.get(key) {
            None => return None,
            Some((_, ttl)) => ttl.is_expired(),
        };

        if should_remove {
            self.storage.remove(key);
            None
        } else {
            self.storage.get(key).map(|(v, _)| v.as_ref())
        }
    }

    pub fn set(&mut self, key: String, value: String, px: Option<u64>) {
        let ttl = px
            .map(|v| Utc::now().timestamp_millis() as u64 + v)
            .map_or(Ttl::None, Ttl::Milliseconds);
        self.storage.insert(key, (value, ttl));
    }

    pub fn get_keys(&mut self) -> Vec<&str> {
        let to_delete: Vec<String> = self
            .storage
            .iter()
            .filter(|(_, (_, ttl))| ttl.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        for key in to_delete {
            self.storage.remove(&key);
        }

        self.storage.keys().map(|x| x.as_str()).collect()
    }

    pub fn backup_database(&self, path: &Path) -> Result<(), RedisStorageError> {
        let mut file = File::create(path).map_err(|e| RedisStorageError {
            msg: format!("error creating file: {}", e),
        })?;
        let db = self
            .storage
            .iter()
            .filter(|(_, (_, ttl))| !ttl.is_expired())
            .map(|(k, (v, ttl))| (k.as_str(), (v.as_str(), ttl)))
            .collect();
        let data = vec![(1, db)];
        write_database("0001", None, &data, &mut file, false).map_err(|e| RedisStorageError {
            msg: format!("error backup database: {}", e),
        })
    }
}
#[derive(thiserror::Error, Debug)]
pub struct RedisStorageError {
    msg: String,
}

impl Display for RedisStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
