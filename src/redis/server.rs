use crate::redis::core::{Configuration, RequestHandler};
use crate::redis::rdb::RedisStorage;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
        let mut request_handler = RequestHandler::new(storage, self.configuration.clone());

        let mut poll = Poll::new().unwrap();
        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            self.configuration.port(),
        );
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

                        if request_handler.handle_request(stream).is_err() {
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
    let mut storage = RedisStorage::default();
    if let Some(path) = configuration.get_db_file_path() {
        let result = storage.restore_database(&path);
        if let Err(e) = result {
            log::error!("error restoring storage: {}", e);
        }
    }
    storage
}
