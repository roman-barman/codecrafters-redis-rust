use crate::redis::core::Storage;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct RedisStorage {
    storage: HashMap<String, (String, Option<i64>)>,
}

impl RedisStorage {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl Storage for RedisStorage {
    fn get(&mut self, key: &str) -> Option<&str> {
        let should_remove = match self.storage.get(key) {
            None => return None,
            Some((_, ttl)) => is_expired(ttl),
        };

        if should_remove {
            self.storage.remove(key);
            None
        } else {
            self.storage.get(key).map(|(v, _)| v.as_ref())
        }
    }

    fn set(&mut self, key: String, value: String, px: Option<i64>) {
        let ttl = px.map(|v| Utc::now().timestamp_millis() + v);
        self.storage.insert(key, (value, ttl));
    }

    fn get_keys(&mut self) -> Vec<&str> {
        let to_delete: Vec<String> = self
            .storage
            .iter()
            .filter(|(_, (_, ttl))| is_expired(ttl))
            .map(|(k, _)| k.clone())
            .collect();

        for key in to_delete {
            self.storage.remove(&key);
        }

        self.storage.keys().map(|x| x.as_str()).collect()
    }
}

fn is_expired(ttl: &Option<i64>) -> bool {
    ttl.is_some() && DateTime::from_timestamp_millis(ttl.unwrap()).unwrap() <= Utc::now()
}
