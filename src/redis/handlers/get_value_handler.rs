use crate::redis::request::Request;
use crate::redis::response::Response;
use crate::redis::server::GetStorage;
use crate::redis::Server;
use anyhow::Error;
use thiserror::Error;

pub trait GetValueHandler: GetStorage {
    fn get_value(&mut self, request: &Request) -> Result<Response, Error> {
        if request.len() != 2 {
            Err(GetValueHandlerError::WrongNumberOfArguments.into())
        } else {
            let key = request.get(1).unwrap();
            let result = self.get_storage().get(key);
            Ok(Response::BulkString(result.map(|x| x.to_string())))
        }
    }
}

impl GetValueHandler for Server {}

#[derive(Error, Debug)]
pub enum GetValueHandlerError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
}
