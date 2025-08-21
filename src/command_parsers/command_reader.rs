use crate::command_parsers::command_parser::CommandParser;
use crate::command_parsers::commands::Commands;
use crate::resp::RespType;
use anyhow::{anyhow, Error};

pub struct CommandReader {
    parsers: Vec<Box<dyn CommandParser>>,
}

impl CommandReader {
    pub fn new() -> Self {
        CommandReader {
            parsers: vec![]
        }
    }

    pub fn register<TParser>(&mut self, parser: TParser)
    where
        TParser: CommandParser + 'static,
    {
        self.parsers.push(Box::new(parser));
    }

    pub fn read<'a>(&self, resp: &'a RespType) -> Result<Commands<'a>, Error> {
        for parser in self.parsers.iter() {
            if parser.can_parse(resp) {
                return parser.parse(resp);
            }
        }
        Err(anyhow!("No parser found"))
    }
}

