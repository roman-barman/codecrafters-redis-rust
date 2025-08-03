use crate::resp::RespType;
use std::collections::VecDeque;

pub(crate) struct EchoCommand;

impl EchoCommand {
    pub(crate) fn execute(&mut self, args: &mut VecDeque<RespType>) -> RespType {
        let arg = args.pop_front().unwrap_or(RespType::BulkString("".to_string()));
        if arg.is_string() {
            RespType::BulkString(arg.get_string_value().unwrap())
        } else {
            RespType::Error("ECHO requires a string argument.".to_string())
        }
    }
}
