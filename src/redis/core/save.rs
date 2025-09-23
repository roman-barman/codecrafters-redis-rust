use crate::redis::core::response::Response;
use crate::redis::rdb::RedisStorage;
use crate::redis::Configuration;

pub fn save(storage: &mut RedisStorage, configuration: &Configuration) -> Response {
    if let Some(path) = configuration.get_db_file_path() {
        let _ = storage.backup_database(&path);
    }

    Response::SimpleString("OK".to_string())
}
