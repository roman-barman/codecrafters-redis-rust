use crate::redis::handlers::{
    EchoHandlerError, GetConfigError, GetValueHandlerError, SetKeyValueHandlerError,
};
use crate::redis::message_reader::MessageReaderError;
use crate::redis::request::RequestError;
use std::io::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("{0}")]
    Client(String),
    #[error("{0}")]
    Connection(String),
}

impl From<MessageReaderError> for RedisError {
    fn from(value: MessageReaderError) -> Self {
        match value {
            MessageReaderError::Io(e) => RedisError::Connection(e.to_string()),
            _ => RedisError::Client(value.to_string()),
        }
    }
}

impl From<RequestError> for RedisError {
    fn from(value: RequestError) -> Self {
        match value {
            RequestError::EmptyRequest => RedisError::Connection(value.to_string()),
            RequestError::InvalidRequest => RedisError::Client(value.to_string()),
        }
    }
}

impl From<GetConfigError> for RedisError {
    fn from(value: GetConfigError) -> Self {
        RedisError::Client(value.to_string())
    }
}

impl From<EchoHandlerError> for RedisError {
    fn from(value: EchoHandlerError) -> Self {
        match value {
            EchoHandlerError::WrongNumberOfArguments => RedisError::Client(value.to_string()),
        }
    }
}

impl From<GetValueHandlerError> for RedisError {
    fn from(value: GetValueHandlerError) -> Self {
        match value {
            GetValueHandlerError::WrongNumberOfArguments => RedisError::Client(value.to_string()),
        }
    }
}

impl From<Error> for RedisError {
    fn from(value: Error) -> Self {
        RedisError::Connection(value.to_string())
    }
}

impl From<SetKeyValueHandlerError> for RedisError {
    fn from(value: SetKeyValueHandlerError) -> Self {
        RedisError::Client(value.to_string())
    }
}
