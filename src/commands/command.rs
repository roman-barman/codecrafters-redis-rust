use crate::resp::RespType;

pub trait Command {
    fn execute(&mut self) -> Result<RespType, String>;
}
