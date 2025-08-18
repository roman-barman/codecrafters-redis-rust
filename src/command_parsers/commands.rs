pub enum Commands<'a> {
    Ping,
    Echo(&'a str),
    Get(&'a str),
}
