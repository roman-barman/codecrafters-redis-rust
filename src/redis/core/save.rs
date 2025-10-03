use crate::redis::core::WriteResp;
use crate::redis::rdb::RedisStorage;
use crate::redis::Configuration;

pub fn save(
    writer: &mut impl WriteResp,
    storage: &mut RedisStorage,
    configuration: &Configuration,
) -> std::io::Result<()> {
    if let Some(path) = configuration.get_db_file_path() {
        let _ = storage.backup_database(&path);
    }
    writer.write_simple_string("OK")
}
