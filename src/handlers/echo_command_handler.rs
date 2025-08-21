use crate::commands::EchoCommand;
use crate::handlers::CommandHandler;
use anyhow::Error;

pub struct EchoCommandHandler;

impl EchoCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler<EchoCommand, String> for EchoCommandHandler {
    fn handle(&mut self, command: &EchoCommand) -> Result<String, Error> {
        Ok(command.as_ref().to_string())
    }
}
