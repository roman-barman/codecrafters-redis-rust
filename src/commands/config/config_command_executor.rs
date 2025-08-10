use crate::commands::config::get::GetConfigCommand;
use crate::config::Config;
use crate::resp::RespType;
use std::collections::VecDeque;
use std::sync::Arc;

const GET: &str = "GET";

pub struct ConfigCommandExecutor {
    config: Arc<Config>,
}

impl ConfigCommandExecutor {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config
        }
    }

    pub fn execute(&self, request: &mut VecDeque<RespType>) -> RespType {
        if request.is_empty() {
            return RespType::Error("No command".to_string());
        }

        let command = request.pop_front().unwrap();
        if command.is_string() {
            self.run_command(command.get_string_value().unwrap().as_str(), request)
        } else {
            RespType::Error("Invalid command. Expected bulk string or simple string.".to_string())
        }
    }

    fn run_command(&self, command: &str, args: &mut VecDeque<RespType>) -> RespType {
        match command {
            GET => {
                let command = GetConfigCommand::new(self.config.clone());
                command.execute(args)
            }
            _ => RespType::Error(format!("Unknown command: {}", command))
        }
    }
}
