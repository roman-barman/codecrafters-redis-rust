use crate::redis::core::{Configuration, Error, RequestHandler};
use crate::redis::storage::RedisStorage;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

const LISTENER_TOKEN: Token = Token(0);

pub struct Server {
    configuration: Rc<Configuration>,
}

impl Server {
    pub fn new(configuration: Configuration) -> Self {
        Self {
            configuration: Rc::new(configuration),
        }
    }

    pub fn run(&mut self) {
        log::info!("Starting server");
        let storage = create_storage(&self.configuration);
        let mut request_handler =
            RequestHandler::new(Box::new(storage), self.configuration.clone());

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

                        if let Err(Error::Connection(_)) = request_handler.handle_request(stream) {
                            poll.registry().deregister(stream).unwrap();
                            connections.remove(&token);
                        }
                    }
                }
            }
        }
    }
}

fn create_storage(configuration: &Configuration) -> RedisStorage {
    let dir = configuration.dir();
    let db_file_name = configuration.db_file_name();
    let mut storage = RedisStorage::default();

    if let Some(dir) = dir {
        if let Some(db_file_name) = db_file_name {
            let path = Path::new(dir).join(db_file_name);
            let result = storage.restore(&path);
            if let Err(e) = result {
                log::error!("error restoring storage: {}", e);
            }
        }
    }

    storage
}
