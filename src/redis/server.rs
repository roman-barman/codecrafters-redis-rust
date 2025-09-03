use crate::config::Config;
use crate::redis::handlers::{EchoHandler, GetConfigHandler, GetValueHandler, PingHandler, SetKeyValueHandler};
use crate::redis::message_reader::MessageReader;
use crate::redis::message_writer::MessageWriter;
use crate::redis::redis_error::RedisError;
use crate::redis::response::Response;
use crate::redis::storage::{KeySettings, RedisStorage, Storage};
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
            "set" => {
                if request.len() < 3 {
                    Err(RedisError::Client(
                        "set: wrong number of arguments".to_string(),
                    ))
                } else {
                    let key = request.get(1).unwrap();
                    let value = request.get(2).unwrap();
                    let settings = match request.get(3) {
                        None => Ok(KeySettings::default()),
                        Some(value) => {
                            let arg_name = value.to_lowercase();
                            if "px" != arg_name {
                                Err(RedisError::Client(format!(
                                    "set: unknown argument name '{}'",
                                    value
                                )))
                            } else {
                                let arg_value = request.get(4);
                                if arg_value.is_none() {
                                    Err(RedisError::Client(
                                        "set: wrong number of arguments".to_string(),
                                    ))
                                } else {
                                    let ttl = arg_value.unwrap().parse::<u64>();
                                    match ttl {
                                        Ok(ttl) => Ok(KeySettings::new(ttl)),
                                        Err(_) => Err(RedisError::Client(
                                            "set: invalid px value".to_string(),
                                        )),
                                    }
                                }
                            }
                        }
                    }?;

                    let result = self.set_key_value(key.clone(), value.clone(), settings);
                    Ok(Response::BulkString(Some(result)))
                }
            }
            "config" => {
                if request.len() != 3 {
                    Err(RedisError::Client(
                        "config: wrong number of arguments".to_string(),
                    ))
                } else {
                    let arg = request.get(1).unwrap();
                    if arg.to_lowercase() != "get" {
                        Err(RedisError::Client(format!(
                            "get: unknown argument name '{}'",
                            arg
                        )))
                    } else {
                        let (key, value) =
                            self.get_config(request.get(2).unwrap(), &self.config)?;

                        Ok(Response::Array(vec![
                            Some(key.to_string()),
                            value.map(|x| x.to_string()),
                        ]))
                    }
                }
            }
            _ => Err(RedisError::Client(format!("Unknown command '{}'", command))),
        }
    }
}

impl GetStorage for Server {
    fn get_storage(&mut self) -> &mut dyn Storage {
        &mut self.storage
    }
}
