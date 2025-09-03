use crate::redis::Server;

pub trait EchoHandler {
    fn echo(&self, msg: &str) -> String {
        msg.to_string()
    }
}

impl EchoHandler for Server {}
