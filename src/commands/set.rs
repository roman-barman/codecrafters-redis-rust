use crate::storages::Storage;
use std::sync::{Arc, Mutex};

pub(crate) struct SetCommand {
    storage: Arc<Mutex<dyn Storage>>,
}

impl SetCommand {
    pub(crate) fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }

    pub(crate) fn execute(&mut self, key: &str, value: &str) -> String {
        self.storage.lock().unwrap().set(key, value);
        String::from("OK")
    }
}
