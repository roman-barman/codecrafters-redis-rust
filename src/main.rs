use crate::cli_args::CliArgs;
use crate::commands::CommandExecutor;
use crate::config::Config;
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

fn main() {
    println!("Logs from your program will appear here!");

    let args = CliArgs::parse();
    let config = Config::from(args);

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let pool = ThreadPool::new(4);
    let storage = Arc::new(Mutex::new(HashMapStorage::new()));
    let command_executor = Arc::new(CommandExecutor::new(storage, Arc::new(config)));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let cloned_command_executor = command_executor.clone();
                pool.execute(|| {
                    handle_client(stream, cloned_command_executor);
                })
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, command_executor: Arc<CommandExecutor>) {
    let mut buffer = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();

        if bytes_read == 0 {
            break;
        }

        let request = std::str::from_utf8(&buffer[..bytes_read]).unwrap().trim();
        println!("request: {}", request);
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
