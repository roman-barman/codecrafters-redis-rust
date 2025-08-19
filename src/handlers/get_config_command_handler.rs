use crate::commands::{Command, GetConfigCommand};
use crate::config::Config;
use crate::handlers::CommandHandler;
use anyhow::Error;
use std::sync::Arc;

const DIR: &str = "dir";
const DB_FILE_NAME: &str = "dbfilename";

pub struct GetConfigCommandHandler {
    configuration: Arc<Config>,
}

impl GetConfigCommandHandler {
    pub fn new(configuration: Arc<Config>) -> Self {
        Self { configuration }
    }
}

impl CommandHandler<GetConfigCommand, String> for GetConfigCommandHandler {
    fn handle(&self, command: &GetConfigCommand) -> Result<String, Error>
    where
        GetConfigCommand: Command<String>,
    {
        let arg = command.as_ref();
        if arg.eq_ignore_ascii_case(DIR) {
            Ok(format!("{} {}", DIR, self.configuration.dir.as_ref().unwrap_or(&"".to_string())))
        } else if arg.eq_ignore_ascii_case(DB_FILE_NAME) {
            Ok(format!("{} {}", DB_FILE_NAME, self.configuration.dbfilename.as_ref().unwrap_or(&"".to_string())))
        } else {
            Err(anyhow::anyhow!("{} unknown argument", arg))
        }
    }
}
