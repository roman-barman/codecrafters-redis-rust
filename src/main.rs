use crate::commands::CommandExecutor;
use crate::thread_pool::ThreadPool;
use std::io::{Read, Write};
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
        println!("request: {}", request);
        let result = CommandExecutor::execute(request);
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
