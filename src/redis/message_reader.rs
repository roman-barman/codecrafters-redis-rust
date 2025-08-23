use anyhow::Error;
use mio::net::TcpStream;
use std::io::{BufRead, BufReader, Read};
use std::str::FromStr;

pub trait MessageReader: Read {
    fn read_message(&self) -> Result<Vec<Option<String>>, Error>;
}

impl MessageReader for TcpStream {
    fn read_message(&self) -> Result<Vec<Option<String>>, Error> {
        let reader = BufReader::with_capacity(10, self);
        read_message(reader)
    }
}

fn read_message(reader: impl BufRead) -> Result<Vec<Option<String>>, Error> {
    let mut lines = Vec::new();
    let mut result_size = 1;
    let mut previous_marker = None;
    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            match RespType::from_str(line?.as_str())? {
                RespType::Array(size) => {
                    result_size = size;
                    previous_marker = Some(RespType::Array(size));
                }
                RespType::BulkString(size) => {
                    if size == -1 {
                        lines.push(None);
                    } else {
                        previous_marker = Some(RespType::BulkString(size));
                    }
                }
                RespType::Integer(s) => lines.push(Some(s)),
                RespType::SimpleString(s) => lines.push(Some(s)),
                _ => return Err(Error::msg("Unexpected RESP data type"))
            }
            continue;
        }

        if lines.len() >= result_size {
            break;
        }

        if let Some(RespType::BulkString(size)) = previous_marker {
            match line {
                Err(e) => return Err(e.into()),
                Ok(line) => {
                    if line.len() != size as usize {
                        return Err(Error::msg("Invalid bulk string length"));
                    }
                    lines.push(Some(line));
                    previous_marker = None;
                }
            }
            continue;
        }

        match RespType::from_str(line?.as_str())? {
            RespType::BulkString(size) => {
                if size == -1 {
                    lines.push(None);
                } else {
                    previous_marker = Some(RespType::BulkString(size));
                }
            }
            RespType::Integer(s) => lines.push(Some(s)),
            RespType::SimpleString(s) => lines.push(Some(s)),
            _ => return Err(Error::msg("Unexpected RESP data type"))
        }
    }
    Ok(lines)
}

enum RespType {
    SimpleString(String),
    BulkString(i64),
    Array(usize),
    Error,
    Integer(String),
}

impl FromStr for RespType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        match chars.next() {
            Some('+') => Ok(RespType::SimpleString(chars.as_str().to_string())),
            Some('$') => {
                let size = chars.as_str().parse::<i64>()?;
                Ok(RespType::BulkString(size))
            }
            Some('-') => Ok(RespType::Error),
            Some(':') => Ok(RespType::Integer(chars.as_str().to_string())),
            Some('*') => {
                let size = chars.as_str().parse::<usize>()?;
                Ok(RespType::Array(size))
            }
            _ => Err(Error::msg("Unknown RESP data type"))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::redis::message_reader::read_message;
    use std::io;

    #[test]
    fn test_read_integer() {
        assert_eq!(
            read_message(io::Cursor::new(b":1000\r\n")).unwrap(),
            vec![Some("1000".to_string())]);
    }

    #[test]
    fn test_read_simple_string() {
        assert_eq!(
            read_message(io::Cursor::new(b"+OK\r\n")).unwrap(),
            vec![Some("OK".to_string())]);
    }

    #[test]
    fn test_read_bulk_string() {
        assert_eq!(
            read_message(io::Cursor::new(b"$5\r\nhello\r\n")).unwrap(),
            vec![Some("hello".to_string())]);
    }

    #[test]
    fn test_read_empty_bulk_string() {
        assert_eq!(
            read_message(io::Cursor::new(b"$0\r\n\r\n")).unwrap(),
            vec![Some("".to_string())]);
    }

    #[test]
    fn test_read_null_bulk_string() {
        assert_eq!(
            read_message(io::Cursor::new(b"$-1\r\n\r\n")).unwrap(),
            vec![None]);
    }

    #[test]
    fn test_read_array() {
        assert_eq!(
            read_message(io::Cursor::new(b"*2\r\n$4\r\nECHO\r\n$5\r\nmango\r\n")).unwrap(),
            vec![Some("ECHO".to_string()), Some("mango".to_string())]);
    }

    #[test]
    fn test_read_empty_stream() {
        assert_eq!(
            read_message(io::Cursor::new(b"")).unwrap(),
            vec![]);
    }
}
