use crate::storages::storage::Storage;
use crate::storages::storage_value::{KeySettings, StorageValue};
use std::collections::HashMap;

pub struct HashMapStorage {
    storage: HashMap<String, StorageValue>,
}

impl HashMapStorage {
    pub fn new() -> HashMapStorage {
        HashMapStorage {
            storage: HashMap::new(),
        }
    }
}

impl Storage for HashMapStorage {
    fn get(&mut self, key: &str) -> Option<&str> {
        let should_remove = match self.storage.get(key) {
            None => return None,
            Some(v) => v.is_value_expired()
        };

        println!("should remove: {}", should_remove);

        if should_remove {
            self.storage.remove(key);
            None
        } else {
            self.storage.get(key).map(|v| v.as_ref())
        }
    }

    fn set(&mut self, key: &str, value: &str, key_settings: KeySettings) {
        let value = StorageValue::new(value.to_string(), key_settings);
        self.storage.insert(key.to_string(), value);
    }
}
