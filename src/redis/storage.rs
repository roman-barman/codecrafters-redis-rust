use chrono::{DateTime, Local, TimeDelta};
use std::collections::HashMap;

pub trait Storage {
    fn get(&mut self, key: &str) -> Option<&str>;
    fn set(&mut self, key: String, value: String, key_settings: KeySettings);
}

pub struct RedisStorage {
    storage: HashMap<String, (String, KeySettings)>,
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
            Some((_, settings)) => settings.ttl.is_some() && settings.ttl.unwrap() <= Local::now(),
        };

        if should_remove {
            self.storage.remove(key);
            None
        } else {
            self.storage.get(key).map(|(v, _)| v.as_ref())
        }
    }

    fn set(&mut self, key: String, value: String, key_settings: KeySettings) {
        self.storage.insert(key, (value, key_settings));
    }
}

pub struct KeySettings {
    ttl: Option<DateTime<Local>>,
}

impl Default for KeySettings {
    fn default() -> Self {
        Self { ttl: None }
    }
}

impl KeySettings {
    pub fn new(expiry: u64) -> Self {
        Self {
            ttl: Some(Local::now() + TimeDelta::try_milliseconds(expiry as i64).unwrap()),
        }
    }
}
