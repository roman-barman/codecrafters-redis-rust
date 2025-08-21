use crate::core::{CommandParser, Commands};
use crate::resp::RespType;
use anyhow::Error;

const COMMAND_NAME: &str = "config";
const GET_COMMAND_NAME: &str = "get";

pub struct GetConfigCommandParser;

impl CommandParser for GetConfigCommandParser {
    fn parse<'a>(&self, command: &'a RespType) -> Result<Commands<'a>, Error> {
        if !self.can_parse(command) {
            return Err(anyhow::anyhow!("Invalid command"));
        }

        match command {
            RespType::Array(array) => {
                if array.len() == 3 {
                    let value = array.get(2).unwrap();
                    match value {
                        RespType::SimpleString(s) => Ok(Commands::GetConfig(s.as_str())),
                        RespType::BulkString(s) => Ok(Commands::GetConfig(s.as_str())),
                        _ => Err(anyhow::anyhow!(
                            "Unexpected RESP type for CONFIG GET command argument"
                        )),
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Unexpected arguments number for CONFIG GET command"
                    ))
                }
            }
            _ => Err(anyhow::anyhow!(
                "Unexpected RESP type for CONFIG GET command"
            )),
        }
    }

    fn can_parse(&self, command: &RespType) -> bool {
        match command {
            RespType::Array(array) => {
                if array.len() >= 2 {
                    let command = array.get(0).unwrap();
                    let is_command = match command {
                        RespType::BulkString(command) => is_config(command),
                        RespType::SimpleString(command) => is_config(command),
                        _ => false,
                    };

                    let sub_command = array.get(1).unwrap();
                    let is_sub_command = match sub_command {
                        RespType::BulkString(command) => is_get(command),
                        RespType::SimpleString(command) => is_get(command),
                        _ => false,
                    };

                    is_command && is_sub_command
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

fn is_config(command: &str) -> bool {
    command.eq_ignore_ascii_case(COMMAND_NAME)
}

fn is_get(command: &str) -> bool {
    command.eq_ignore_ascii_case(GET_COMMAND_NAME)
}
