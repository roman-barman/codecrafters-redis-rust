use crate::commands::command::Command;
use crate::resp::RespType;

pub struct PingCommand();

impl Command for PingCommand {
    fn execute(&mut self) -> Result<RespType, String> {
        Ok(RespType::SimpleString("PONG".to_string()))
    }
}
