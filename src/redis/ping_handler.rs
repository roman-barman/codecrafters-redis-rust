use crate::redis::Server;

pub trait PingHandler {
    fn ping(&self) -> String {
        "PONG".to_string()
    }
}

impl PingHandler for Server {}
