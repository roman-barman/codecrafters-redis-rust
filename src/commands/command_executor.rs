use crate::commands::config::ConfigCommandExecutor;
use crate::commands::get::GetCommand;
use crate::commands::set::SetCommand;
use crate::config::Config;
use crate::resp::RespType;
use crate::storages::Storage;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const SET: &str = "SET";
const GET: &str = "GET";
const CONFIG: &str = "CONFIG";

pub struct CommandExecutor {
    storage: Arc<Mutex<dyn Storage>>,
    config_command_executor: ConfigCommandExecutor,
}

impl CommandExecutor {
    pub fn new(storage: Arc<Mutex<dyn Storage>>, config: Arc<Config>) -> Self {
        Self { storage, config_command_executor: ConfigCommandExecutor::new(config) }
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
                    match command.as_str() {
                        CONFIG => self.config_command_executor.execute(&mut array),
                        _ => {
                            if array.is_empty() {
                                self.run_command_without_args(command.as_str())
                            } else {
                                self.run_command_with_args(command.as_str(), &mut array)
                            }
                        }
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
            _ => RespType::Error(format!("Unknown command: {}", command))
        }
    }

    fn run_command_with_args(&self, command: &str, args: &mut VecDeque<RespType>) -> RespType {
        match command {
            SET => {
                let mut command = SetCommand::new(self.storage.clone());
                command.execute(args)
            }
            GET => {
                let command = GetCommand::new(self.storage.clone());
                command.execute(args)
            }
            _ => RespType::Error(format!("Unknown command: {}", command))
        }
    }
}
