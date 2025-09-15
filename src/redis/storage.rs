use crate::redis::core::Storage;
use crate::redis::rdb::Ttl;
use chrono::Utc;
use std::collections::HashMap;

pub struct RedisStorage {
    storage: HashMap<String, (String, Ttl)>,
}

impl RedisStorage {
    pub fn new(data: HashMap<String, (String, Ttl)>) -> Self {
        Self {
            storage: data
                .into_iter()
                .filter(|(_, (_, ttl))| !ttl.is_expired())
                .collect(),
        }
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
}
