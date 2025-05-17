use crate::commands::Command;
use std::io::Write;
use std::net::TcpStream;

pub struct EchoCommand<'a, 'b> {
    stream: &'a mut TcpStream,
    arg: &'b str,
}

impl<'a, 'b> EchoCommand<'a, 'b> {
    pub fn new(stream: &'a mut TcpStream, arg: &'b str) -> Self {
        Self { stream, arg }
    }
}

impl Command for EchoCommand<'_, '_> {
    fn execute(&mut self) {
        self.stream.write_all(self.arg.as_bytes()).unwrap();
    }
}
