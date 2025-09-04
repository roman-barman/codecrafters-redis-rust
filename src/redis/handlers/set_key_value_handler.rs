use crate::redis::request::Request;
use crate::redis::response::Response;
use crate::redis::server::GetStorage;
use crate::redis::storage::KeySettings;
use crate::redis::Server;
use thiserror::Error;

pub trait SetKeyValueHandler: GetStorage {
    fn set_key_value(&mut self, request: &Request) -> Result<Response, SetKeyValueHandlerError> {
        if request.len() < 3 {
            return Err(SetKeyValueHandlerError::WrongNumberOfArguments);
        }

        let key = request.get(1).unwrap().to_string();
        let value = request.get(2).unwrap().to_string();
        let settings = match request.get(3) {
            None => KeySettings::default(),
            Some(value) => {
                let arg_name = value.to_lowercase();
                if "px" != arg_name {
                    return Err(SetKeyValueHandlerError::UnknownArgument(value.to_string()));
                }

                let arg_value = request.get(4);
                if arg_value.is_none() {
                    return Err(SetKeyValueHandlerError::WrongNumberOfArguments);
                }

                let ttl = arg_value.unwrap();
                match ttl.parse::<u64>() {
                    Ok(ttl) => KeySettings::new(ttl),
                    Err(_) => return Err(SetKeyValueHandlerError::InvalidPxValue(ttl.to_string())),
                }
            }
        };
        self.get_storage().set(key, value, settings);
        Ok(Response::BulkString(Some("OK".to_string())))
    }
}

impl SetKeyValueHandler for Server {}

#[derive(Error, Debug)]
pub enum SetKeyValueHandlerError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
    #[error("unknown argument: '{0}'")]
    UnknownArgument(String),
    #[error("invalid px value: '{0}'")]
    InvalidPxValue(String),
}
