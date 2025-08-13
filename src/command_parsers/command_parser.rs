use crate::command_parsers::Commands;
use crate::resp::RespType;
use anyhow::Error;

pub trait CommandParser: Sync + Send {
    fn parse(&self, command: &RespType) -> Result<Commands, Error>;
    fn can_parse(&self, command: &RespType) -> bool;
}