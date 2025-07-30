use crate::commands::command::Command;

pub struct PingCommand;

impl Command<String> for PingCommand {
    fn execute(&mut self) -> Result<String, String> {
        Ok("PONG".to_string())
    }
}
