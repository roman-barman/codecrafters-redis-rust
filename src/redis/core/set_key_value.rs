use crate::redis::core::request::Request;
use crate::redis::core::WriteResponse;
use crate::redis::rdb::RedisStorage;

const OK: Option<&str> = Some("OK");

pub fn set_key_value(
    writer: &mut impl WriteResponse,
    storage: &mut RedisStorage,
    request: &Request,
) -> std::io::Result<()> {
    if request.len() < 3 {
        return writer.write_error("wrong number of arguments for 'set' command");
    }

    let key = request.get(1).unwrap().to_string();
    let value = request.get(2).unwrap().to_string();
    let px = match request.get(3) {
        None => None,
        Some(value) => {
            let arg_name = value.to_lowercase();
            if "px" != arg_name {
                return writer.write_error(format!("unknown argument: '{}'", value));
            }

            let arg_value = request.get(4);
            if arg_value.is_none() {
                return writer.write_error("wrong number of arguments for 'set' command");
            }

            let px = arg_value.unwrap();
            match px.parse::<u64>() {
                Ok(px) => Some(px),
                Err(_) => return writer.write_error(format!("invalid px value: '{}'", value)),
            }
        }
    };
    storage.set(key, value, px);
    writer.write_bulk_sting(&OK)
}
