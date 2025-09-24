use crate::redis::core::WriteResponse;
use crate::redis::rdb::RedisStorage;

pub fn get_keys(
    writer: &mut impl WriteResponse,
    storage: &mut RedisStorage,
) -> std::io::Result<()> {
    let result: Vec<_> = storage.get_keys().into_iter().map(Some).collect();
    writer.write_array(&result)
}
