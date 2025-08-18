use crate::commands::Command;

pub struct GetCommand {
    key: String,
}

impl GetCommand {
    pub fn new(key: String) -> Self {
        Self {
            key
        }
    }
}

impl Command<Option<String>> for GetCommand {}

impl AsRef<str> for GetCommand {
    fn as_ref(&self) -> &str {
        &self.key
    }
}
