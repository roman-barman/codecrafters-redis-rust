use crate::config::Config;
use crate::redis::echo_handler::EchoHandler;
use crate::redis::get_config_handler::GetConfigHandler;
use crate::redis::get_value_handler::GetValueHandler;
use crate::redis::message_reader::MessageReader;
use crate::redis::message_writer::MessageWriter;
use crate::redis::ping_handler::PingHandler;
use crate::redis::set_key_value_handler::SetKeyValueHandler;
use crate::redis::storage::{KeySettings, RedisStorage, Storage};
use anyhow::Error;
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
        Self { storage: RedisStorage::new(), config }
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

                        if !self.handle_request(stream) {
                            poll.registry().deregister(stream).unwrap();
                            connections.remove(&token);
                        }
                    }
                }
            }
        }
    }

    fn handle_request(&mut self, stream: &mut TcpStream) -> bool {
        let request = stream.read_message();
        let request = match request {
            Ok(request) => request,
            Err(e) => {
                println!("error: {}", e);
                return false;
            }
        };

        if request.is_empty() {
            return false;
        }

        if request.iter().any(|x| x.is_none()) {
            return match stream.write_error("Invalid argument type") {
                Ok(_) => true,
                Err(e) => {
                    println!("error: {}", e);
                    false
                }
            };
        }

        let command = request.get(0).unwrap().as_ref().unwrap().to_lowercase();
        let command = command.as_str();
        let result = match command {
            "ping" => Ok(RespResponse::SimpleString(self.ping())),
            "echo" => {
                if request.len() != 2 {
                    Ok(RespResponse::Error("Wrong number of arguments".to_string()))
                } else {
                    Ok(RespResponse::BulkString(Some(self.echo(request.get(1).unwrap().as_ref().unwrap()))))
                }
            }
            "get" => {
                if request.len() != 2 {
                    Ok(RespResponse::Error("Wrong number of arguments".to_string()))
                } else {
                    let result = self.get_value(request.get(1).unwrap().as_ref().unwrap());
                    Ok(RespResponse::BulkString(result.map(|x| x.to_string())))
                }
            }
            "set" => {
                if request.len() < 3 {
                    Ok(RespResponse::Error("Wrong number of arguments".to_string()))
                } else {
                    let key = request.get(1).unwrap().as_ref().unwrap();
                    let value = request.get(2).unwrap().as_ref().unwrap();
                    let settings = match request.get(3) {
                        None => Ok(KeySettings::default()),
                        Some(value) => {
                            let arg_name = value.as_ref().unwrap().to_lowercase();
                            if "px" != arg_name {
                                Err("Unknown argument name")
                            } else {
                                let arg_value = request.get(4);
                                if arg_value.is_none() {
                                    Err("Wrong number of arguments")
                                } else {
                                    let ttl = arg_value.unwrap().as_ref().unwrap().parse::<u64>();
                                    match ttl {
                                        Ok(ttl) => Ok(KeySettings::new(ttl)),
                                        Err(_) => Err("Invalid argument value")
                                    }
                                }
                            }
                        }
                    };

                    match settings {
                        Ok(settings) => {
                            let result = self.set_key_value(
                                key.clone(),
                                value.clone(), settings);
                            Ok(RespResponse::BulkString(Some(result)))
                        }
                        Err(e) => Ok(RespResponse::Error(e.to_string()))
                    }
                }
            }
            "config" => {
                if request.len() != 3 {
                    Ok(RespResponse::Error("Wrong number of arguments".to_string()))
                } else {
                    if request.get(1).unwrap().as_ref().unwrap().to_lowercase() != "get" {
                        Ok(RespResponse::Error("Unknown argument name".to_string()))
                    } else {
                        let result = self.get_config(
                            request.get(2).unwrap().as_ref().unwrap(), &self.config);

                        match result {
                            Ok((key, value)) => Ok(RespResponse::Array(
                                vec![
                                    Some(key.to_string()),
                                    value.map(|x| x.to_string())])),
                            Err(e) => Ok(RespResponse::Error(e.to_string()))
                        }
                    }
                }
            }
            _ => Err(Error::msg("Unknown command")),
        };

        match result {
            Ok(response) => match response {
                RespResponse::BulkString(s) => stream
                    .write_bulk_sting(&s)
                    .map_or(false, |_| true),
                RespResponse::SimpleString(s) => stream
                    .write_simple_string(s)
                    .map_or(false, |_| true),
                RespResponse::Error(s) => stream
                    .write_error(s)
                    .map_or(false, |_| true),
                RespResponse::Array(s) => stream
                    .write_array(&s)
                    .map_or(false, |_| true),
            },
            Err(e) => {
                println!("error: {}", e);
                false
            }
        }
    }
}

impl GetStorage for Server {
    fn get_storage(&mut self) -> &mut dyn Storage {
        &mut self.storage
    }
}

enum RespResponse {
    SimpleString(String),
    BulkString(Option<String>),
    Array(Vec<Option<String>>),
    Error(String),
}
