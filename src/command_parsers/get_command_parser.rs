use crate::core::{CommandParser, Commands};
use crate::resp::RespType;
use anyhow::Error;

const COMMAND_NAME: &str = "get";

pub struct GetCommandParser;

impl CommandParser for GetCommandParser {
    fn parse<'a>(&self, command: &'a RespType) -> Result<Commands<'a>, Error> {
        if !self.can_parse(command) {
            return Err(anyhow::anyhow!("Invalid command"));
        }

        match command {
            RespType::Array(array) => {
                if array.len() == 2 {
                    let value = array.get(1).unwrap();
                    match value {
                        RespType::SimpleString(s) => Ok(Commands::Get(s.as_str())),
                        RespType::BulkString(s) => Ok(Commands::Get(s.as_str())),
                        _ => Err(anyhow::anyhow!(
                            "Unexpected RESP type for GET command argument"
                        )),
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Unexpected arguments number for GET command"
                    ))
                }
            }
            _ => Err(anyhow::anyhow!("Unexpected RESP type for GET command")),
        }
    }

    fn can_parse(&self, command: &RespType) -> bool {
        match command {
            RespType::Array(array) => {
                if !array.is_empty() {
                    let command = array.get(0).unwrap();
                    match command {
                        RespType::BulkString(command) => is_get(command),
                        RespType::SimpleString(command) => is_get(command),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

fn is_get(command: &str) -> bool {
    command.eq_ignore_ascii_case(COMMAND_NAME)
}
