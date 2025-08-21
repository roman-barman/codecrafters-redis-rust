use crate::core::{CommandParser, Commands};
use crate::resp::RespType;
use anyhow::Error;

const COMMAND_NAME: &str = "echo";

pub struct EchoCommandParser;

impl CommandParser for EchoCommandParser {
    fn parse<'a>(&self, command: &'a RespType) -> Result<Commands<'a>, Error> {
        if !self.can_parse(command) {
            return Err(anyhow::anyhow!("Invalid command"));
        }

        match command {
            RespType::SimpleString(_) => Ok(Commands::Echo("")),
            RespType::BulkString(_) => Ok(Commands::Echo("")),
            RespType::Array(array) => {
                if array.len() == 2 {
                    let value = array.get(1).unwrap();
                    match value {
                        RespType::SimpleString(s) => Ok(Commands::Echo(s.as_str())),
                        RespType::BulkString(s) => Ok(Commands::Echo(s.as_str())),
                        _ => Err(anyhow::anyhow!(
                            "Unexpected RESP type for ECHO command argument"
                        )),
                    }
                } else {
                    Ok(Commands::Echo(""))
                }
            }
            _ => Err(anyhow::anyhow!("Unexpected RESP type for ECHO command")),
        }
    }

    fn can_parse(&self, command: &RespType) -> bool {
        match command {
            RespType::SimpleString(s) => is_echo(s.as_str()),
            RespType::BulkString(s) => is_echo(s.as_str()),
            RespType::Array(array) => {
                if !array.is_empty() && array.len() <= 2 {
                    let command = array.get(0).unwrap();
                    self.can_parse(command)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

fn is_echo(command: &str) -> bool {
    command.eq_ignore_ascii_case(COMMAND_NAME)
}
