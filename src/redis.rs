use crate::command_parsers::{CommandReader, EchoCommandParser, GetCommandParser, GetConfigCommandParser, PingCommandParser, SetCommandParser};
use crate::config::Config;
use crate::engine::Engine;
use crate::handlers::{EchoCommandHandler, GetCommandHandler, GetConfigCommandHandler, PingCommandHandler, SetCommandHandler};
use crate::mediators::Mediator;
use crate::storages::HashMapStorage;
use anyhow::Error;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::rc::Rc;

const LISTENER_TOKEN: Token = Token(0);

pub struct Redis {
    engine: Engine,
}

impl Redis {
    pub fn new(config: Config) -> Self {
        let storage = Rc::new(RefCell::new(HashMapStorage::new()));

        let mut mediator = Mediator::new();
        mediator.register(PingCommandHandler::new());
        mediator.register(EchoCommandHandler::new());
        mediator.register(GetCommandHandler::new(storage.clone()));
        mediator.register(SetCommandHandler::new(storage.clone()));
        mediator.register(GetConfigCommandHandler::new(config));

        let mut command_reader = CommandReader::new();
        command_reader.register(Box::new(PingCommandParser));
        command_reader.register(Box::new(EchoCommandParser));
        command_reader.register(Box::new(GetCommandParser));
        command_reader.register(Box::new(SetCommandParser));
        command_reader.register(Box::new(GetConfigCommandParser));

        let engine = Engine::new(mediator, command_reader);
        Self {
            engine
        }
    }

    pub fn run(&self) {
        let mut poll = Poll::new().unwrap();
        let addr = "127.0.0.1:6379".parse().unwrap();
        let mut listener = TcpListener::bind(addr).unwrap();

        poll.registry().register(&mut listener, LISTENER_TOKEN, Interest::READABLE).unwrap();

        let mut events = Events::with_capacity(1024);
        let mut connections = HashMap::new();
        let mut next_token = Token(1);

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    LISTENER_TOKEN => {
                        let (mut stream, _) = listener.accept().unwrap();
                        let token = next_token;
                        next_token.0 += 1;
                        poll.registry().register(&mut stream, token, Interest::READABLE | Interest::WRITABLE).unwrap();
                        connections.insert(token, stream);
                    }
                    token => {
                        let stream = connections.get_mut(&token).unwrap();
                        if event.is_readable() {
                            let mut buffer = [0; 512];
                            match stream.read(&mut buffer) {
                                Ok(0) => {
                                    poll.registry().deregister(stream).unwrap();
                                    connections.remove(&token);
                                }
                                Ok(n) => {
                                    if event.is_writable() {
                                        match self.handle(&buffer[..n]) {
                                            Ok(response) => {
                                                stream.write(response.as_bytes()).unwrap();
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                                Err(e) => println!("error: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle(&self, buffer: &[u8]) -> Result<String, Error> {
        let request = std::str::from_utf8(buffer)?.trim();
        println!("request: {}", request);

        match self.engine.handle_request(request) {
            Ok(result) => {
                let result: String = result.into();
                println!("response: {}", result);
                Ok(result)
            }
            Err(e) => {
                println!("error: {}", e);
                Err(e)
            }
        }
    }
}
