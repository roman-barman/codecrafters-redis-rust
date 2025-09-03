pub enum Response {
    SimpleString(String),
    BulkString(Option<String>),
    Array(Vec<Option<String>>),
}