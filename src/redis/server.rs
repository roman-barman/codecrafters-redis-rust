use crate::redis::client::TcpClient;
use crate::redis::core::{Configuration, RequestHandler};
use crate::redis::rdb::RedisStorage;
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;
use std::str::FromStr;

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
        let storage = self.create_storage();
        let mut request_handler = RequestHandler::new(storage, self.configuration.clone());

        if let Some(addr) = self.configuration.replicaof() {
            if !self.replicaof_handshake(addr) {
                return;
            }
        }

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

    fn replicaof_handshake(&self, addr: &str) -> bool {
        let addr = addr.split_once(' ');
        match addr {
            Some((address, port)) => {
                let address = if address == "localhost" {
                    "127.0.0.1"
                } else {
                    address
                };
                let port = port.parse::<u16>();
                if port.is_err() {
                    log::error!("Invalid replicaof port format");
                    return false;
                }
                match handshake(address, port.unwrap(), self.configuration.port()) {
                    Ok(_) => {
                        log::info!("replicaof handshake successful");
                        true
                    }
                    Err(e) => {
                        log::error!("replicaof handshake failed: {}", e);
                        false
                    }
                }
            }
            None => {
                log::error!("Invalid replicaof configuration format");
                false
            }
        }
    }

    fn create_storage(&self) -> RedisStorage {
        let mut storage = RedisStorage::default();
        if let Some(path) = self.configuration.get_db_file_path() {
            let result = storage.restore_database(&path);
            if let Err(e) = result {
                log::error!("error restoring storage: {}", e);
            }
        }
        storage
    }
}

fn handshake(address: &str, master_port: u16, slave_port: u16) -> std::io::Result<()> {
    log::debug!(
        "handshake: connecting to master at {}:{}",
        address,
        master_port
    );
    let addr = SocketAddr::from_str(format!("{}:{}", address, master_port).as_str()).unwrap();
    let mut client = TcpClient::connect(addr)?;

    log::info!("handshake send: PING");
    client.send(&[Some("PING")])?;
    match client.receive() {
        Ok(response) => {
            if response.len() != 1 || response.first().unwrap().as_str() != "PONG" {
                log::error!("handshake failed: invalid response");
                return Err(std::io::Error::other("handshake failed"));
            }

            log::info!("handshake received: PONG");
        }
        Err(e) => {
            log::error!("handshake failed: {}", e);
            return Err(std::io::Error::other("handshake failed"));
        }
    }

    log::info!("handshake send: REPLCONF listening-port {}", slave_port);
    client.send(&[
        Some("REPLCONF"),
        Some("listening-port"),
        Some(slave_port.to_string().as_str()),
    ])?;
    receive_replconf_ack(&mut client)?;

    log::info!("handshake send: REPLCONF capa psync2",);
    client.send(&[Some("REPLCONF"), Some("capa"), Some("psync2")])?;
    receive_replconf_ack(&mut client)?;

    log::info!("handshake send: PSYNC ? -1",);
    client.send(&[Some("PSYNC"), Some("?"), Some("-1")])?;
    let _ = client.receive();

    Ok(())
}

fn receive_replconf_ack(client: &mut TcpClient) -> std::io::Result<()> {
    match client.receive() {
        Ok(response) => {
            if response.len() != 1 || response.first().unwrap().as_str() != "OK" {
                log::error!("handshake failed: invalid response");
                return Err(std::io::Error::other("handshake failed"));
            }

            log::info!("handshake received: OK");
            Ok(())
        }
        Err(e) => {
            log::error!("handshake failed: {}", e);
            Err(std::io::Error::other("handshake failed"))
        }
    }
}
