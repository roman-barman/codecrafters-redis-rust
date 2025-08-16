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

    pub fn register(&mut self, parser: Box<dyn CommandParser>) {
        self.parsers.push(parser);
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

