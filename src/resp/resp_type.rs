#[derive(Debug, PartialEq)]
pub enum RespType {
    SimpleString(String),
    BulkString(String),
    Error(String),
}
