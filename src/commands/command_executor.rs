use crate::commands::command::Command;
use crate::commands::echo::EchoCommand;
use crate::commands::ping::PingCommand;
use crate::resp::RespType;
use std::collections::VecDeque;

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute(command: &str) -> Result<String, String> {
        let resp_result = RespType::try_from(command);

        match resp_result {
            Err(e) => {
                println!("Error: {}", e);
                Err(RespType::Error(e.to_string()).into())
            }
            Ok(resp_type) => {
                let result: String = run_command(resp_type).into();
                println!("Result: {}", result);
                Ok(result)
            }
        }
    }
}

fn run_command(resp: RespType) -> RespType {
    match resp {
        RespType::BulkString(command) => run_command_without_args(command.as_str()),
        RespType::SimpleString(command) => run_command_without_args(command.as_str()),
        RespType::Array(mut array) => {
            let command = array.pop_front().unwrap();

            if command.is_string() {
                let command = command.get_string_value().unwrap();
                if array.is_empty() {
                    run_command_without_args(command.as_str())
                } else {
                    run_command_with_args(command.as_str(), &mut array)
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

fn run_command_without_args(command: &str) -> RespType {
    match command {
        "PING" => {
            let mut command = PingCommand;
            RespType::SimpleString(command.execute().unwrap())
        }
        _ => RespType::Error(format!("Unknown command: {}", command))
    }
}

fn run_command_with_args(command: &str, args: &mut VecDeque<RespType>) -> RespType {
    match command {
        "ECHO" => {
            let arg = args.pop_front().unwrap_or(RespType::BulkString("".to_string()));
            if arg.is_string() {
                let mut command = EchoCommand::new(arg.get_string_value().unwrap());
                RespType::BulkString(command.execute().unwrap())
            } else {
                RespType::Error("ECHO requires a string argument.".to_string())
            }
        }
        _ => RespType::Error(format!("Unknown command: {}", command))
    }
}
