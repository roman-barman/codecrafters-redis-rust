use crate::commands::command::Command;
use std::io::Write;
use std::net::TcpStream;

pub struct PingCommand<'a> {
    stream: &'a mut TcpStream,
}

impl<'a> PingCommand<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self { stream }
    }
}

impl Command for PingCommand<'_> {
    fn execute(&mut self) {
        self.stream.write_all(b"+PONG\r\n").unwrap();
    }
}
