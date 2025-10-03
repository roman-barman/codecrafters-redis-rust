use crate::redis::core::{ReadResp, WriteResp};
use crate::redis::reader::MessageReaderError;
use mio::net::TcpStream;
use mio::{Events, Interest, Poll, Token};
use std::io::{self};
use std::net::SocketAddr;
use std::time::Duration;

const CLIENT_TOKEN: Token = Token(0);

pub struct TcpClient {
    stream: TcpStream,
    poll: Poll,
}

impl TcpClient {
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
        let mut stream = TcpStream::connect(addr)?;
        let poll = Poll::new()?;

        poll.registry().register(
            &mut stream,
            CLIENT_TOKEN,
            Interest::READABLE | Interest::WRITABLE,
        )?;

        Ok(Self { stream, poll })
    }

    pub fn send(&mut self, data: &[Option<impl AsRef<str>>]) -> io::Result<()> {
        self.stream.write_array(data)
    }

    pub fn receive(&mut self) -> Result<Vec<String>, MessageReaderError> {
        self.wait_for_readable()?;
        self.stream.read_resp()
    }

    fn wait_for_readable(&mut self) -> io::Result<()> {
        let mut events = Events::with_capacity(1);

        loop {
            self.poll.poll(&mut events, Some(Duration::from_secs(5)))?;

            for event in events.iter() {
                if event.token() == CLIENT_TOKEN && event.is_readable() {
                    return Ok(());
                }
            }
        }
    }
}
