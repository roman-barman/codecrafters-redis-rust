use crate::redis::core::response::Response;

pub fn ping() -> Response {
    Response::SimpleString("PONG".to_string())
}
