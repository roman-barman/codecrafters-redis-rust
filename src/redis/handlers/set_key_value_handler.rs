use crate::redis::server::GetStorage;
use crate::redis::storage::KeySettings;
use crate::redis::Server;

pub trait SetKeyValueHandler: GetStorage {
    fn set_key_value(&mut self, key: String, value: String, key_settings: KeySettings) -> String {
        self.get_storage().set(key, value, key_settings);
        "OK".to_string()
    }
}

impl SetKeyValueHandler for Server {}
