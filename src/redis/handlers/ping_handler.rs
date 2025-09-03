use crate::redis::response::Response;
use crate::redis::Server;

pub trait PingHandler {
    fn ping(&self) -> Response {
        Response::SimpleString("PONG".to_string())
    }
}

impl PingHandler for Server {}
