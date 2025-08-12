use crate::commands::PingCommand;
use crate::handlers::command_handler::CommandHandler;

pub struct PingCommandHandler;

impl PingCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler<PingCommand, String> for PingCommandHandler {
    fn handle(&self, _command: PingCommand) -> Result<String, anyhow::Error> {
        Ok("PONG".to_string())
    }
}
