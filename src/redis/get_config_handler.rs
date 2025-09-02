use crate::config::Config;
use crate::redis::Server;
use anyhow::Error;
use thiserror::Error;

const DIR: &str = "dir";
const DB_FILE_NAME: &str = "dbfilename";

pub trait GetConfigHandler<'a> {
    fn get_config(
        &self,
        parameter: &str,
        config: &'a Config,
    ) -> Result<(&'static str, Option<&'a str>), Error> {
        if parameter.eq_ignore_ascii_case(DIR) {
            Ok((DIR, config.dir.as_ref().map(|d| d.as_str())))
        } else if parameter.eq_ignore_ascii_case(DB_FILE_NAME) {
            Ok((DB_FILE_NAME, config.dbfilename.as_ref().map(|d| d.as_str())))
        } else {
            Err(GetConfigError::UnknownParameter(parameter.to_string()).into())
        }
    }
}

#[derive(Debug, Error)]
pub enum GetConfigError {
    #[error("unknown configuration parameter: '{0}'")]
    UnknownParameter(String),
}

impl<'a> GetConfigHandler<'a> for Server {}
