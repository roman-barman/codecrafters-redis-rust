use crate::redis::core::request::Request;
use crate::redis::core::WriteResp;
use crate::redis::rdb::RedisStorage;

pub fn get_value(
    writer: &mut impl WriteResp,
    storage: &mut RedisStorage,
    request: &Request,
) -> std::io::Result<()> {
    if request.len() != 2 {
        writer.write_error("wrong number of arguments for 'get' command")
    } else {
        let key = request.get(1).unwrap();
        let result = storage.get(key);
        writer.write_bulk_sting(&result)
    }
}
