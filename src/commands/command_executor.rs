use crate::commands::echo::EchoCommand;
use crate::commands::get::GetCommand;
use crate::commands::ping::PingCommand;
use crate::commands::set::SetCommand;
use crate::resp::RespType;
use crate::storages::Storage;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const ECHO: &str = "ECHO";
const PING: &str = "PING";
const SET: &str = "SET";
const GET: &str = "GET";

pub struct CommandExecutor {
    storage: Arc<Mutex<dyn Storage>>,
}

impl CommandExecutor {
    pub fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }
    pub fn execute(&self, command: &str) -> Result<String, String> {
        let resp_result = RespType::try_from(command);

        match resp_result {
            Err(e) => {
                println!("Error: {}", e);
                Err(RespType::Error(e.to_string()).into())
            }
            Ok(resp_type) => {
                let result: String = self.run_command(resp_type).into();
                println!("Result: {}", result);
                Ok(result)
            }
        }
    }

    fn run_command(&self, resp: RespType) -> RespType {
        match resp {
            RespType::BulkString(command) => self.run_command_without_args(command.as_str()),
            RespType::SimpleString(command) => self.run_command_without_args(command.as_str()),
            RespType::Array(mut array) => {
                let command = array.pop_front().unwrap();

                if command.is_string() {
                    let command = command.get_string_value().unwrap();
                    if array.is_empty() {
                        self.run_command_without_args(command.as_str())
                    } else {
                        self.run_command_with_args(command.as_str(), &mut array)
                    }
                } else {
                    RespType::Error("Invalid command. Expected bulk string or simple string.".to_string())
                }
            }
            _ => RespType::Error(
                "Invalid command. Expected bulk string or simple string.".to_string()
            )
        }
    }

    fn run_command_without_args(&self, command: &str) -> RespType {
        match command {
            PING => {
                let mut command = PingCommand;
                RespType::SimpleString(command.execute())
            }
            _ => RespType::Error(format!("Unknown command: {}", command))
        }
    }

    fn run_command_with_args(&self, command: &str, args: &mut VecDeque<RespType>) -> RespType {
        match command {
            ECHO => {
                let mut command = EchoCommand;
                command.execute(args)
            }
            SET => {
                if args.len() != 2 {
                    return RespType::Error(format!("{} requires 2 arguments.", SET));
                }

                let key = args.pop_front().unwrap();
                let value = args.pop_front().unwrap();

                if !key.is_string() || !value.is_string() {
                    return RespType::Error(format!("{} requires string arguments.", SET));
                }

                let mut command = SetCommand::new(self.storage.clone());
                RespType::SimpleString(
                    command.execute(key.get_string_value().unwrap().as_str(),
                                    value.get_string_value().unwrap().as_str()))
            }
            GET => {
                if args.len() != 1 {
                    return RespType::Error(format!("{} requires 1 arguments.", GET));
                }

                let key = args.pop_front().unwrap();
                if !key.is_string() {
                    return RespType::Error(format!("{} requires string argument.", GET));
                }
                let command = GetCommand::new(self.storage.clone());
                let result = command.execute(key.get_string_value().unwrap().as_str());
                match result {
                    Some(value) => RespType::BulkString(value),
                    None => RespType::NullBulkString
                }
            }
            _ => RespType::Error(format!("Unknown command: {}", command))
        }
    }
}
