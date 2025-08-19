use crate::command_parsers::{CommandReader, Commands};
use crate::commands::{EchoCommand, GetCommand, GetConfigCommand, PingCommand, SetCommand};
use crate::mediators::Mediator;
use crate::resp::RespType;

pub struct Engine {
    mediator: Mediator,
    command_reader: CommandReader,
}

impl Engine {
    pub fn new(mediator: Mediator, command_reader: CommandReader) -> Self {
        Self {
            mediator,
            command_reader,

        }
    }

    pub fn handle_request(&self, request: &str) -> Result<RespType, anyhow::Error> {
        let resp_result = RespType::try_from(request)?;
        let command = self.command_reader.read(&resp_result);

        match command {
            Ok(c) => {
                match c {
                    Commands::Ping => {
                        self.mediator.send(Box::new(PingCommand)).map(|x| RespType::SimpleString(x))
                    }
                    Commands::Echo(s) => {
                        self.mediator.send(Box::new(EchoCommand::new(s.to_string()))).map(|x| RespType::BulkString(x))
                    }
                    Commands::Get(s) => {
                        self.mediator.send(Box::new(GetCommand::new(s.to_string())))
                            .map(|x| x.map_or(RespType::NullBulkString, |s| RespType::BulkString(s)))
                    }
                    Commands::Set(arg) => {
                        self.mediator.send(Box::new(SetCommand::new(arg.key.to_string(), arg.value.to_string(), arg.expiry)))
                            .map(|x| RespType::BulkString(x))
                    }
                    Commands::GetConfig(arg) => {
                        self.mediator.send(Box::new(GetConfigCommand::new(arg.to_string())))
                            .map(|x| RespType::Array(x.split(' ').map(|x| RespType::BulkString(x.to_string())).collect()))
                    }
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}