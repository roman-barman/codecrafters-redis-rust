use crate::redis::core::request::Request;
use crate::redis::core::response::Response;
use thiserror::Error;

pub fn echo(request: &Request) -> Result<Response, EchoError> {
    if request.len() != 2 {
        Err(EchoError::WrongNumberOfArguments)
    } else {
        Ok(Response::BulkString(Some(
            request.get(1).unwrap().to_string(),
        )))
    }
}

#[derive(Error, Debug)]
pub enum EchoError {
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
}
