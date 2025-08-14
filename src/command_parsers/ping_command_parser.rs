use crate::command_parsers::command_parser::CommandParser;
use crate::command_parsers::Commands;
use crate::resp::RespType;
use anyhow::Error;

const COMMAND_NAME: &str = "PING";

pub struct PingCommandParser;

impl CommandParser for PingCommandParser {
    fn parse(&self, command: &RespType) -> Result<Commands, Error> {
        if !self.can_parse(command) {
            Err(anyhow::anyhow!("Invalid command"))
        } else {
            Ok(Commands::Ping)
        }
    }

    fn can_parse(&self, command: &RespType) -> bool {
        match command {
            RespType::SimpleString(s) => is_ping(s.as_str()),
            RespType::BulkString(s) => is_ping(s.as_str()),
            RespType::Array(array) => {
                if array.len() == 1 {
                    let command = array.get(0).unwrap();
                    self.can_parse(command)
                } else {
                    false
                }
            }
            _ => false
        }
    }
}

fn is_ping(command: &str) -> bool {
    command == COMMAND_NAME
}
