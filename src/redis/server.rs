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
                        println!("start");
                        let stream = connections.get_mut(&token).unwrap();
                        println!("is_readable: {}", event.is_readable());
                        println!("is_writable: {}", event.is_writable());
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
        println!("handle");
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
            "ping" => stream.write_simple_string(self.ping().as_str()),
            "echo" => {
                if request.len() != 2 {
                    stream.write_error("Wrong number of arguments")
                } else {
                    stream.write_bulk_sting(Some(self.echo(request.get(1).unwrap().as_ref().unwrap()).as_str()))
                }
            }
            "get" => {
                if request.len() != 2 {
                    stream.write_error("Wrong number of arguments")
                } else {
                    let result = self.get_value(request.get(1).unwrap().as_ref().unwrap());
                    stream.write_bulk_sting(result)
                }
            }
            "set" => {
                println!("0");
                if request.len() < 3 {
                    stream.write_error("Wrong number of arguments")
                } else {
                    println!("a");
                    let key = request.get(1).unwrap().as_ref().unwrap();
                    let value = request.get(2).unwrap().as_ref().unwrap();
                    println!("b");
                    let settings = match request.get(3) {
                        None => Ok(KeySettings::default()),
                        Some(value) => {
                            let arg_name = value.as_ref().unwrap().to_lowercase();
                            if "px" != arg_name {
                                println!("c");
                                Err("Unknown argument name")
                            } else {
                                println!("d");
                                let arg_value = request.get(4);
                                println!("e");
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
                            stream.write_bulk_sting(Some(result.as_str()))
                        }
                        Err(e) => stream.write_error(e)
                    }
                }
            }
            "config" => {
                if request.len() != 3 {
                    stream.write_error("Wrong number of arguments")
                } else {
                    if request.get(1).unwrap().as_ref().unwrap().to_lowercase() != "get" {
                        stream.write_error("Unknown argument name")
                    } else {
                        let result = self.get_config(
                            request.get(2).unwrap().as_ref().unwrap(), &self.config);

                        match result {
                            Ok((key, value)) => stream.write_array(vec![Some(key), value]),
                            Err(e) => stream.write_error(e.to_string().as_str())
                        }
                    }
                }
            }
            _ => Err(Error::msg("Unknown command")),
        };

        match result {
            Ok(_) => true,
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
