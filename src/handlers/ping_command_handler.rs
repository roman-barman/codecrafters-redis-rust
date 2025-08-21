use crate::commands::PingCommand;
use crate::core::CommandHandler;

pub struct PingCommandHandler;

impl PingCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler<PingCommand, String> for PingCommandHandler {
    fn handle(&mut self, _command: &PingCommand) -> Result<String, anyhow::Error> {
        Ok("PONG".to_string())
    }
}
