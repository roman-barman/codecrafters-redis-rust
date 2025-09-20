use crate::redis::core::response::Response;
use crate::redis::rdb::RedisStorage;

pub fn get_keys(storage: &mut RedisStorage) -> Response {
    Response::Array(
        storage
            .get_keys()
            .into_iter()
            .map(|x| Some(x.to_string()))
            .collect(),
    )
}
