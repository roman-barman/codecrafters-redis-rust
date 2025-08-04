use crate::storages::storage_value::KeySettings;

pub trait Storage: Send + Sync {
    fn get(&mut self, key: &str) -> Option<&str>;
    fn set(&mut self, key: &str, value: &str, key_settings: KeySettings);
}