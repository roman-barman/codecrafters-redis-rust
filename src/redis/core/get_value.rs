use crate::redis::core::request::Request;
use crate::redis::core::response::Response;
use crate::redis::core::storage::Storage;
use thiserror::Error;

pub fn get_value(
    storage: &mut Box<dyn Storage>,
    request: &Request,
) -> Result<Response, GetValueError> {
    if request.len() != 2 {
        Err(GetValueError::WrongNumberOfArguments)
    } else {
        let key = request.get(1).unwrap();
        let result = storage.get(key);
        Ok(Response::BulkString(result.map(|x| x.to_string())))
    }
}

#[derive(Error, Debug)]
pub enum GetValueError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
}
