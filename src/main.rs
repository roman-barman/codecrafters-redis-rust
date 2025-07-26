use crate::commands::PingCommand;
use crate::commands::{Command, EchoCommand};
use crate::thread_pool::ThreadPool;
use std::io::Read;
use std::net::{TcpListener, TcpStream};

mod thread_pool;
mod commands;
mod resp;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                pool.execute(|| {
                    handle_client(stream);
                })
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();

        if bytes_read == 0 {
            break;
        }

        let request = std::str::from_utf8(&buffer[..bytes_read]).unwrap().trim();
        let index = request.find(' ');
        let command = &request[..index.unwrap_or(bytes_read)];

        let args = if index.is_some() {
            request[index.unwrap() + 1..].trim().split_whitespace().collect::<Vec<&str>>()
        } else {
            Vec::new()
        };

        match command {
            "PING" => {
                let mut command = PingCommand::new(&mut stream);
                command.execute();
            }
            "ECHO" => {
                let arg = args.get(0).unwrap_or(&"");
                let mut command = EchoCommand::new(&mut stream, arg);
                command.execute();
            }
            _ => {
                println!("unknown command");
            }
        }
    }
}
