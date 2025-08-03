use crate::storages::storage::Storage;
use std::collections::HashMap;

pub struct HashMapStorage {
    storage: HashMap<String, String>,
}

impl HashMapStorage {
    pub fn new() -> HashMapStorage {
        HashMapStorage {
            storage: HashMap::new(),
        }
    }
}

impl Storage for HashMapStorage {
    fn get(&self, key: &str) -> Option<&str> {
        self.storage.get(key).map(|v| v.as_str())
    }

    fn set(&mut self, key: &str, value: &str) {
        self.storage.insert(key.to_string(), value.to_string());
    }
}
