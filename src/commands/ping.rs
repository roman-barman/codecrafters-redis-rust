use crate::resp::RespType;

pub(crate) struct PingCommand;

impl PingCommand {
    pub(crate) fn execute(&self) -> RespType {
        RespType::SimpleString("PONG".to_string())
    }
}
