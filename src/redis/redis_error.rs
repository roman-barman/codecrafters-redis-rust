use crate::redis::get_config_handler::GetConfigError;
use crate::redis::message_reader::MessageReaderError;
use crate::redis::request::RequestError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("{0}")]
    ServerError(String),
    #[error("{0}")]
    ClientError(String),
    #[error("{0}")]
    ConnectionError(String),
}

impl From<anyhow::Error> for RedisError {
    fn from(value: anyhow::Error) -> Self {
        if value.is::<MessageReaderError>() {
            return RedisError::from(value.downcast::<MessageReaderError>().unwrap());
        }

        if value.is::<RequestError>() {
            return RedisError::from(value.downcast::<RequestError>().unwrap());
        }

        if value.is::<GetConfigError>() {
            return RedisError::from(value.downcast::<GetConfigError>().unwrap());
        }

        if value.is::<std::io::Error>() {
            return RedisError::ConnectionError(value.to_string());
        }

        RedisError::ServerError(value.to_string())
    }
}

impl From<MessageReaderError> for RedisError {
    fn from(value: MessageReaderError) -> Self {
        match value {
            MessageReaderError::Io(e) => RedisError::ConnectionError(e.to_string()),
            _ => RedisError::ClientError(value.to_string()),
        }
    }
}

impl From<RequestError> for RedisError {
    fn from(value: RequestError) -> Self {
        match value {
            RequestError::EmptyRequest => {
                RedisError::ConnectionError(RequestError::EmptyRequest.to_string())
            }
            RequestError::InvalidRequest => {
                RedisError::ClientError(RequestError::InvalidRequest.to_string())
            }
        }
    }
}

impl From<GetConfigError> for RedisError {
    fn from(value: GetConfigError) -> Self {
        match value {
            GetConfigError::UnknownParameter(parameter) => {
                RedisError::ClientError(GetConfigError::UnknownParameter(parameter).to_string())
            }
        }
    }
}
