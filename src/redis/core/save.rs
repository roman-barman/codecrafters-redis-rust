use crate::redis::core::response::Response;
use crate::redis::core::Storage;
use crate::redis::Configuration;
use std::path::{Path, PathBuf};

pub fn save(storage: &mut Box<dyn Storage>, configuration: &Configuration) -> Response {
    let dir = configuration.dir();
    let db_file_name = configuration.db_file_name();
    if dir.is_some() && db_file_name.is_some() {
        let path = PathBuf::from(Path::new(dir.unwrap()).join(db_file_name.unwrap()));
        let _ = storage.save(&path);
    }
    Response::SimpleString("OK".to_string())
}
