use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;

pub trait Storage {
    fn get(&mut self, key: &str) -> Option<&str>;
    fn set(&mut self, key: String, value: String, px: Option<i64>);
}

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
            Some((_, ttl)) => {
                if ttl.is_none() {
                    false
                } else {
                    DateTime::from_timestamp_millis(ttl.unwrap()).unwrap() <= Utc::now()
                }
            }
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
}
