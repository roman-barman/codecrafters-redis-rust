use crate::config::Config;
use crate::redis::request::Request;
use crate::redis::response::Response;
use crate::redis::Server;
use thiserror::Error;

const DIR: &str = "dir";
const DB_FILE_NAME: &str = "dbfilename";

pub trait GetConfigHandler {
    fn get_config(&self, request: &Request, config: &Config) -> Result<Response, GetConfigError> {
        if request.len() != 3 {
            return Err(GetConfigError::WrongNumberOfArguments);
        }

        let arg = request.get(1).unwrap();
        if arg.to_lowercase() != "get" {
            return Err(GetConfigError::UnknownArgumentName(arg.to_string()));
        }

        let parameter = request.get(2).unwrap();

        if parameter.eq_ignore_ascii_case(DIR) {
            Ok(Response::Array(vec![
                Some(DIR.to_string()),
                config.dir.as_deref().map(|x| x.to_string()),
            ]))
        } else if parameter.eq_ignore_ascii_case(DB_FILE_NAME) {
            Ok(Response::Array(vec![
                Some(DB_FILE_NAME.to_string()),
                config.dbfilename.as_deref().map(|x| x.to_string()),
            ]))
        } else {
            Err(GetConfigError::UnknownParameter(parameter.to_string()))
        }
    }
}

#[derive(Debug, Error)]
pub enum GetConfigError {
    #[error("unknown configuration parameter: '{0}'")]
    UnknownParameter(String),
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
    #[error("unknown argument name: '{0}'")]
    UnknownArgumentName(String),
}

impl GetConfigHandler for Server {}
