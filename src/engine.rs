use crate::command_parsers::{CommandReader, Commands};
use crate::commands::PingCommand;
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
        let command = self.command_reader.read(request);

        match command {
            Ok(c) => {
                match c {
                    Commands::Ping => {
                        self.mediator.send(PingCommand).map(|x| RespType::SimpleString(x))
                    }
                }
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}