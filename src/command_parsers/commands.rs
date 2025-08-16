pub enum Commands<'a> {
    Ping,
    Echo(&'a str),
}
