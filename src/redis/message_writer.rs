use anyhow::Error;
use mio::net::TcpStream;
use std::io::Write;

pub trait MessageWriter: Write {
    fn write_simple_string(&mut self, message: &str) -> Result<(), Error>;
    fn write_error(&mut self, message: &str) -> Result<(), Error>;
    fn write_bulk_sting(&mut self, message: Option<&str>) -> Result<(), Error>;
}

impl MessageWriter for TcpStream {
    fn write_simple_string(&mut self, message: &str) -> Result<(), Error> {
        self.write(format!("+{}\r\n", message).as_bytes())?;
        Ok(())
    }

    fn write_error(&mut self, message: &str) -> Result<(), Error> {
        self.write(format!("-{}\r\n", message).as_bytes())?;
        Ok(())
    }

    fn write_bulk_sting(&mut self, message: Option<&str>) -> Result<(), Error> {
        match message {
            Some(message) => self.write(format!("${}\r\n{}\r\n", message.len(), message).as_bytes())?,
            None => self.write(b"$-1\r\n")?,
        };
        Ok(())
    }
}
