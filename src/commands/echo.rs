use crate::commands::Command;
use crate::resp::RespType;

pub struct EchoCommand {
    arg: String,
}

impl EchoCommand {
    pub fn new(arg: String) -> Self {
        Self { arg }
    }
}

impl Command for EchoCommand {
    fn execute(&mut self) -> Result<RespType, String> {
        Ok(RespType::BulkString(self.arg.clone()))
    }
}
