use crate::storages::Storage;
use std::sync::{Arc, Mutex};

pub(crate) struct GetCommand {
    storage: Arc<Mutex<dyn Storage>>,
}

impl GetCommand {
    pub(crate) fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }

    pub(crate) fn execute(&self, key: &str) -> Option<String> {
        self.storage.lock().unwrap().get(key).map(String::from)
    }
}
