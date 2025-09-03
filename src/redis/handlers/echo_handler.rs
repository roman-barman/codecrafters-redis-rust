use crate::redis::request::Request;
use crate::redis::response::Response;
use crate::redis::Server;
use anyhow::Error;
use thiserror::Error;

pub trait EchoHandler {
    fn echo(&self, request: &Request) -> Result<Response, Error> {
        if request.len() != 2 {
            Err(EchoHandlerError::WrongNumberOfArguments.into())
        } else {
            Ok(Response::BulkString(Some(request.get(1).unwrap().to_string())))
        }
    }
}

impl EchoHandler for Server {}

#[derive(Error, Debug)]
pub enum EchoHandlerError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
}
