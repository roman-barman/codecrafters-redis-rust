use crate::command_parsers::command_parser::CommandParser;
use crate::command_parsers::{Commands, SetArgs};
use crate::resp::RespType;
use anyhow::Error;

const COMMAND_NAME: &str = "set";
const EXPIRY: &str = "px";

pub struct SetCommandParser;

impl CommandParser for SetCommandParser {
    fn parse<'a>(&self, command: &'a RespType) -> Result<Commands<'a>, Error> {
        if !self.can_parse(command) {
            return Err(anyhow::anyhow!("Invalid command"));
        }

        match command {
            RespType::Array(array) => {
                if array.len() >= 3 {
                    let key = match array.get(1).unwrap() {
                        RespType::SimpleString(s) => s.as_str(),
                        RespType::BulkString(s) => s.as_str(),
                        _ => return Err(anyhow::anyhow!("Unexpected RESP type for SET key argument"))
                    };
                    let value = match array.get(2).unwrap() {
                        RespType::SimpleString(s) => s.as_str(),
                        RespType::BulkString(s) => s.as_str(),
                        _ => return Err(anyhow::anyhow!("Unexpected RESP type for SET value argument"))
                    };

                    let mut result = SetArgs {
                        key,
                        value,
                        expiry: None,
                    };

                    let mut i = 3;
                    while i < array.len() {
                        let arg_name = match array.get(i).unwrap() {
                            RespType::SimpleString(s) => s.as_str(),
                            RespType::BulkString(s) => s.as_str(),
                            _ => return Err(anyhow::anyhow!("Unexpected RESP type for SET argument"))
                        };
                        i += 1;

                        if is_expiry(arg_name) {
                            let arg_value = match array.get(i) {
                                None => return Err(anyhow::anyhow!("px argument does not have a value")),
                                Some(RespType::BulkString(s)) => s.as_str(),
                                Some(RespType::SimpleString(s)) => s.as_str(),
                                _ => return Err(anyhow::anyhow!("Unexpected RESP type for SET argument"))
                            };
                            let arg_value = arg_value.parse::<u64>()?;
                            result.expiry = Some(arg_value);

                            i += 1;
                        } else {
                            return Err(anyhow::anyhow!(
                                "Unexpected argument for SET command: {}",
                                arg_name
                            ));
                        }

                        i += 1;
                    }

                    Ok(Commands::Set(result))
                } else {
                    Err(anyhow::anyhow!("Unexpected arguments number for SET command"))
                }
            }
            _ => Err(anyhow::anyhow!("Unexpected RESP type for SET command"))
        }
    }

    fn can_parse(&self, command: &RespType) -> bool {
        match command {
            RespType::Array(array) => {
                if !array.is_empty() {
                    let command = array.get(0).unwrap();
                    match command {
                        RespType::BulkString(command) => is_set(command),
                        RespType::SimpleString(command) => is_set(command),
                        _ => false
                    }
                } else {
                    false
                }
            }
            _ => false
        }
    }
}

fn is_set(command: &str) -> bool {
    command.eq_ignore_ascii_case(COMMAND_NAME)
}

fn is_expiry(arg: &str) -> bool {
    arg.eq_ignore_ascii_case(EXPIRY)
}
