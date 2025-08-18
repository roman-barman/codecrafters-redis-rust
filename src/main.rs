use crate::cli_args::CliArgs;
use crate::command_parsers::{CommandReader, EchoCommandParser, GetCommandParser, PingCommandParser, SetCommandParser};
use crate::commands::CommandExecutor;
use crate::config::Config;
use crate::engine::Engine;
use crate::handlers::{EchoCommandHandler, GetCommandHandler, PingCommandHandler, SetCommandHandler};
use crate::mediators::Mediator;
use crate::storages::HashMapStorage;
use crate::thread_pool::ThreadPool;
use clap::Parser;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

mod thread_pool;
mod commands;
mod resp;
mod storages;
mod cli_args;
mod config;
mod handlers;
mod mediators;
mod command_parsers;
mod engine;

fn main() {
    println!("Logs from your program will appear here!");

    let storage = Arc::new(Mutex::new(HashMapStorage::new()));

    let mut mediator = Mediator::new();
    mediator.register(Box::new(PingCommandHandler::new()));
    mediator.register(Box::new(EchoCommandHandler::new()));
    mediator.register(Box::new(GetCommandHandler::new(storage.clone())));
    mediator.register(Box::new(SetCommandHandler::new(storage.clone())));

    let mut command_reader = CommandReader::new();
    command_reader.register(Box::new(PingCommandParser));
    command_reader.register(Box::new(EchoCommandParser));
    command_reader.register(Box::new(GetCommandParser));
    command_reader.register(Box::new(SetCommandParser));

    let engine = Arc::new(Engine::new(mediator, command_reader));

    let args = CliArgs::parse();
    let config = Config::from(args);

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let pool = ThreadPool::new(4);
    let command_executor = Arc::new(CommandExecutor::new(Arc::new(config)));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let cloned_command_executor = command_executor.clone();
                let engine = engine.clone();
                pool.execute(|| {
                    handle_client(stream, cloned_command_executor, engine);
                })
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, command_executor: Arc<CommandExecutor>, engine: Arc<Engine>) {
    let mut buffer = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();

        if bytes_read == 0 {
            break;
        }

        let request = std::str::from_utf8(&buffer[..bytes_read]).unwrap().trim();
        println!("request: {}", request);
        let result = engine.handle_request(request);
        match result {
            Ok(_) => {
                let result: String = result.unwrap().into();
                stream.write(result.as_bytes()).unwrap();
                continue;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }

        let result = command_executor.execute(request);
        match result {
            Ok(response) => {
                stream.write(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
