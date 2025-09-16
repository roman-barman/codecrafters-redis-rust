use crate::redis::core::Storage;
use crate::redis::rdb::{read_first_database, write_database, Ttl};
use chrono::Utc;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;

pub struct RedisStorage {
    storage: HashMap<String, (String, Ttl)>,
}

impl RedisStorage {
    pub fn restore(&mut self, path: &PathBuf) -> Result<(), RedisStorageError> {
        let mut file = File::open(path).map_err(|_| RedisStorageError {
            msg: format!("error opening file: {}", path.to_str().unwrap()),
        })?;
        let db = read_first_database(&mut file).map_err(|_| RedisStorageError {
            msg: format!("error reading database: {}", path.to_str().unwrap()),
        })?;
        if let Some(db) = db {
            self.storage = db;
        }
        Ok(())
    }
}

impl Default for RedisStorage {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl Storage for RedisStorage {
    fn get(&mut self, key: &str) -> Option<&str> {
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

    fn set(&mut self, key: String, value: String, px: Option<u64>) {
        let ttl = px
            .map(|v| Utc::now().timestamp_millis() as u64 + v)
            .map_or(Ttl::None, |v| Ttl::Milliseconds(v));
        self.storage.insert(key, (value, ttl));
    }

    fn get_keys(&mut self) -> Vec<&str> {
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

    fn save(&self, path: &PathBuf) -> Result<(), Error> {
        let mut file = File::create(path)?;
        let db = self
            .storage
            .iter()
            .filter(|(_, (_, ttl))| !ttl.is_expired())
            .map(|(k, (v, ttl))| (k.as_str(), (v.as_str(), ttl)))
            .collect();
        let mut data = Vec::new();
        data.push((1, db));
        write_database("0001", None, &data, &mut file, false)
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
