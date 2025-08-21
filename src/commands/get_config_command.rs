use crate::commands::Command;

pub struct GetConfigCommand {
    configuration_name: String,
}

impl GetConfigCommand {
    pub fn new(configuration_name: String) -> Self {
        Self { configuration_name }
    }
}

impl Command<String> for GetConfigCommand {}

impl AsRef<str> for GetConfigCommand {
    fn as_ref(&self) -> &str {
        &self.configuration_name
    }
}
