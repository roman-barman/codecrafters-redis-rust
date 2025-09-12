use crate::redis::core::echo::EchoError;
use crate::redis::core::get_config::GetConfigError;
use crate::redis::core::get_value::GetValueError;
use crate::redis::core::set_key_value::SetKeyValueError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Client(String),
    #[error("{0}")]
    Connection(String),
}

impl From<EchoError> for Error {
    fn from(value: EchoError) -> Self {
        Error::Client(value.to_string())
    }
}

impl From<GetValueError> for Error {
    fn from(value: GetValueError) -> Self {
        Error::Client(value.to_string())
    }
}

impl From<SetKeyValueError> for Error {
    fn from(value: SetKeyValueError) -> Self {
        Error::Client(value.to_string())
    }
}

impl From<GetConfigError> for Error {
    fn from(value: GetConfigError) -> Self {
        Error::Client(value.to_string())
    }
}
