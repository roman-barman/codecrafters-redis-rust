use crate::redis::core::request::Request;
use crate::redis::core::response::Response;
use crate::redis::rdb::RedisStorage;
use thiserror::Error;

pub fn set_key_value(
    storage: &mut RedisStorage,
    request: &Request,
) -> Result<Response, SetKeyValueError> {
    if request.len() < 3 {
        return Err(SetKeyValueError::WrongNumberOfArguments);
    }

    let key = request.get(1).unwrap().to_string();
    let value = request.get(2).unwrap().to_string();
    let px = match request.get(3) {
        None => None,
        Some(value) => {
            let arg_name = value.to_lowercase();
            if "px" != arg_name {
                return Err(SetKeyValueError::UnknownArgument(value.to_string()));
            }

            let arg_value = request.get(4);
            if arg_value.is_none() {
                return Err(SetKeyValueError::WrongNumberOfArguments);
            }

            let px = arg_value.unwrap();
            match px.parse::<u64>() {
                Ok(px) => Some(px),
                Err(_) => return Err(SetKeyValueError::InvalidPxValue(px.to_string())),
            }
        }
    };
    storage.set(key, value, px);
    Ok(Response::BulkString(Some("OK".to_string())))
}

#[derive(Error, Debug)]
pub enum SetKeyValueError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
    #[error("unknown argument: '{0}'")]
    UnknownArgument(String),
    #[error("invalid px value: '{0}'")]
    InvalidPxValue(String),
}
