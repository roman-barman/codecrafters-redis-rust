use crate::core::contracts::Commands;
use crate::resp::RespType;
use anyhow::Error;

pub trait CommandParser {
    fn parse<'a>(&self, command: &'a RespType) -> Result<Commands<'a>, Error>;
    fn can_parse(&self, command: &RespType) -> bool;
}
