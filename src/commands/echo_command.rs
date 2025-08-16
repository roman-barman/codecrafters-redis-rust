use crate::commands::Command;

pub struct EchoCommand {
    arg: String,
}

impl EchoCommand {
    pub fn new(arg: String) -> Self {
        Self { arg }
    }
}

impl Command<String> for EchoCommand {}

impl AsRef<str> for EchoCommand {
    fn as_ref(&self) -> &str {
        &self.arg
    }
}
