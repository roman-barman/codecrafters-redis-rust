use crate::commands::command::Command;

pub struct EchoCommand {
    arg: String,
}

impl EchoCommand {
    pub fn new(arg: String) -> Self {
        Self { arg }
    }
}

impl Command<String> for EchoCommand {
    fn execute(&mut self) -> Result<String, String> {
        Ok(self.arg.clone())
    }
}
