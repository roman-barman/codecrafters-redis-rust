use crate::redis::server::GetStorage;
use crate::redis::Server;

pub trait GetValueHandler<'a>: GetStorage {
    fn get_value(&'a mut self, key: &str) -> Option<&'a str> {
        self.get_storage().get(key)
    }
}

impl<'a> GetValueHandler<'a> for Server {}
