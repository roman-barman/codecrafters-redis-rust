pub enum Commands<'a> {
    Ping,
    Echo(&'a str),
    Get(&'a str),
    Set(SetArgs<'a>),
}

pub struct SetArgs<'a> {
    pub key: &'a str,
    pub value: &'a str,
    pub expiry: Option<u64>,
}
