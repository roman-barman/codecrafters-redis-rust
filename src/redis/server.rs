use crate::config::Config;
use crate::redis::handlers::{
    EchoHandler, GetConfigHandler, GetValueHandler, PingHandler, SetKeyValueHandler,
};
use crate::redis::message_reader::MessageReader;
use crate::redis::message_writer::MessageWriter;
use crate::redis::redis_error::RedisError;
use crate::redis::response::Response;
use crate::redis::storage::{RedisStorage, Storage};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;

const LISTENER_TOKEN: Token = Token(0);

pub trait GetStorage {
    fn get_storage(&mut self) -> &mut dyn Storage;
}

pub struct Server {
    storage: RedisStorage,
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            storage: RedisStorage::new(),
            config,
        }
    }

    pub fn run(&mut self) {
        let mut poll = Poll::new().unwrap();
        let addr = "127.0.0.1:6379".parse().unwrap();
        let mut listener = TcpListener::bind(addr).unwrap();

        poll.registry()
            .register(&mut listener, LISTENER_TOKEN, Interest::READABLE)
            .unwrap();

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
                        poll.registry()
                            .register(&mut stream, token, Interest::READABLE | Interest::WRITABLE)
                            .unwrap();
                        connections.insert(token, stream);
                    }
                    token => {
                        let stream = connections.get_mut(&token).unwrap();
                        if !event.is_readable() || !event.is_writable() {
                            continue;
                        }

                        match self.handle(stream) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("{}", e);
                                match e {
                                    RedisError::Client(_) => {}
                                    _ => {
                                        poll.registry().deregister(stream).unwrap();
                                        connections.remove(&token);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle(&mut self, stream: &mut TcpStream) -> Result<(), RedisError> {
        let result = self.handle_request(stream);
        match result {
            Ok(response) => match response {
                Response::SimpleString(value) => Ok(stream.write_simple_string(value)?),
                Response::BulkString(value) => Ok(stream.write_bulk_sting(&value)?),
                Response::Array(value) => Ok(stream.write_array(&value)?),
            },
            Err(e) => {
                println!("{}", e);
                match e {
                    RedisError::Client(e) => Ok(stream.write_error(&e)?),
                    _ => Err(e),
                }
            }
        }
    }

    fn handle_request(&mut self, stream: &mut TcpStream) -> Result<Response, RedisError> {
        let request = stream.read_message()?;
        let binding = request.get(0).unwrap().to_lowercase();
        let command = binding.as_str();
        match command {
            "ping" => Ok(self.ping()),
            "echo" => self.echo(&request).map_err(|e| e.into()),
            "get" => self.get_value(&request).map_err(|e| e.into()),
            "set" => self.set_key_value(&request).map_err(|e| e.into()),
            "config" => self
                .get_config(&request, &self.config)
                .map_err(|e| e.into()),
            _ => Err(RedisError::Client(format!("Unknown command '{}'", command))),
        }
    }
}

impl GetStorage for Server {
    fn get_storage(&mut self) -> &mut dyn Storage {
        &mut self.storage
    }
}
