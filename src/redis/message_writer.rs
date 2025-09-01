use anyhow::Error;
use mio::net::TcpStream;
use std::io::Write;

pub trait MessageWriter: Write {
    fn write_simple_string(&mut self, message: impl AsRef<str>) -> Result<(), Error> {
        self.write(format!("+{}\r\n", message.as_ref()).as_bytes())?;
        Ok(())
    }
    fn write_error(&mut self, message: impl AsRef<str>) -> Result<(), Error> {
        self.write(format!("-{}\r\n", message.as_ref()).as_bytes())?;
        Ok(())
    }
    fn write_bulk_sting(&mut self, message: &Option<impl AsRef<str>>) -> Result<(), Error> {
        match message {
            Some(message) => self.write(format!("${}\r\n{}\r\n", message.as_ref().len(), message.as_ref()).as_bytes())?,
            None => self.write(b"$-1\r\n")?,
        };
        Ok(())
    }

    fn write_array(&mut self, message: &Vec<Option<impl AsRef<str>>>) -> Result<(), Error> {
        self.write(format!("*{}\r\n", message.len()).as_bytes())?;
        for message in message {
            self.write_bulk_sting(message)?;
        }
        Ok(())
    }
}

impl MessageWriter for TcpStream {}
