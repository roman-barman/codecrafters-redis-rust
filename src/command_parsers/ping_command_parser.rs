use crate::command_parsers::command_parser::CommandParser;
use crate::command_parsers::Commands;
use crate::resp::RespType;
use anyhow::Error;

const COMMAND_NAME: &str = "PING";

pub struct PingCommandParser;

impl CommandParser for PingCommandParser {
    fn parse(&self, command: &RespType) -> Result<Commands, Error> {
        match command {
            RespType::SimpleString(s) => parse_ping(s.as_str()),
            RespType::Error(s) => parse_ping(s.as_str()),
            _ => Err(anyhow::anyhow!("Invalid RESP type"))
        }
    }

    fn can_parse(&self, command: &RespType) -> bool {
        match command {
            RespType::SimpleString(s) => is_ping(s.as_str()),
            RespType::Error(s) => is_ping(s.as_str()),
            _ => false
        }
    }
}

fn is_ping(command: &str) -> bool {
    command == COMMAND_NAME
}

fn parse_ping(command: &str) -> Result<Commands, Error> {
    if is_ping(command) {
        Ok(Commands::Ping)
    } else {
        Err(anyhow::anyhow!("Invalid command"))
    }
}
