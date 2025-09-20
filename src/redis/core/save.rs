use crate::redis::core::response::Response;
use crate::redis::rdb::RedisStorage;
use crate::redis::Configuration;
use std::path::Path;

pub fn save(storage: &mut RedisStorage, configuration: &Configuration) -> Response {
    let dir = configuration.dir();
    let db_file_name = configuration.db_file_name();
    if let Some(dir) = dir {
        if let Some(db_file_name) = db_file_name {
            let path = Path::new(dir).join(db_file_name);
            let _ = storage.backup_database(&path);
        }
    }
    Response::SimpleString("OK".to_string())
}
