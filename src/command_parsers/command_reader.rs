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

    pub fn read(&self, request: &str) -> Result<Commands, Error> {
        let resp_result = RespType::try_from(request);
        match resp_result {
            Err(e) => {
                println!("Error: {}", e);
                Err(e.into())
            }
            Ok(resp_type) => {
                self.read_from_resp(&resp_type)
            }
        }
    }

    fn read_from_resp(&self, resp: &RespType) -> Result<Commands, Error> {
        println!("Parsers count: {}", self.parsers.len());
        for parser in self.parsers.iter() {
            if parser.can_parse(resp) {
                return parser.parse(resp);
            }
        }
        Err(anyhow!("No parser found"))
    }
}


