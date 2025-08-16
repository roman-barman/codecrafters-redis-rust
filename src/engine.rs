use crate::command_parsers::{CommandReader, Commands};
use crate::commands::{EchoCommand, PingCommand};
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
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}