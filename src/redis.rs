use crate::command_parsers::{CommandReader, EchoCommandParser, GetCommandParser, GetConfigCommandParser, PingCommandParser, SetCommandParser};
use crate::config::Config;
use crate::engine::Engine;
use crate::handlers::{EchoCommandHandler, GetCommandHandler, GetConfigCommandHandler, PingCommandHandler, SetCommandHandler};
use crate::mediators::Mediator;
use crate::storages::HashMapStorage;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};

pub struct Redis {
    engine: Arc<Engine>,
}

impl Redis {
    pub fn new(config: Config) -> Self {
        let storage = Arc::new(Mutex::new(HashMapStorage::new()));

        let mut mediator = Mediator::new();
        mediator.register(Box::new(PingCommandHandler::new()));
        mediator.register(Box::new(EchoCommandHandler::new()));
        mediator.register(Box::new(GetCommandHandler::new(storage.clone())));
        mediator.register(Box::new(SetCommandHandler::new(storage.clone())));
        mediator.register(Box::new(GetConfigCommandHandler::new(config)));

        let mut command_reader = CommandReader::new();
        command_reader.register(Box::new(PingCommandParser));
        command_reader.register(Box::new(EchoCommandParser));
        command_reader.register(Box::new(GetCommandParser));
        command_reader.register(Box::new(SetCommandParser));
        command_reader.register(Box::new(GetConfigCommandParser));

        let engine = Arc::new(Engine::new(mediator, command_reader));
        Self {
            engine
        }
    }

    pub fn run(&self) {
        let (sender, receiver) = mpsc::channel();
        let engine = self.engine.clone();
        let thread_receiver = std::thread::spawn(move || {
            loop {
                let message = receiver.recv();
                match message {
                    Ok(stream) => {
                        println!("received new connection");
                        handle(stream, engine.clone());
                    }
                    Err(_) => break
                }
            }
        });

        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("accepted new connection");
                    sender.send(stream).unwrap();
                }
                Err(e) => {
                    println!("connection error: {}", e);
                }
            }
        }

        drop(sender);
        thread_receiver.join().unwrap();
    }
}

fn handle(mut stream: TcpStream, engine: Arc<Engine>) {
    let mut buffer = [0; 512];
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(bytes_read) => bytes_read,
            Err(_) => break
        };

        if bytes_read == 0 {
            break;
        }

        let request = std::str::from_utf8(&buffer[..bytes_read]).unwrap().trim();
        println!("request: {}", request);

        match engine.handle_request(request) {
            Ok(result) => {
                let result: String = result.into();
                match stream.write(result.as_bytes()) {
                    Ok(_) => (),
                    Err(_) => break
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
