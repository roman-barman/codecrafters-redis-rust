use crate::redis::core::ReadRequest;
use mio::net::TcpStream;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use thiserror::Error;

impl ReadRequest for TcpStream {
    type Error = MessageReaderError;
    fn read_request(&self) -> Result<Vec<String>, MessageReaderError> {
        let reader = BufReader::with_capacity(10, self);
        Ok(read_message(reader)?)
    }
}

fn read_message(reader: impl BufRead) -> Result<Vec<String>, MessageReaderError> {
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
                        return Err(MessageReaderError::UnknownDataType);
                    } else {
                        previous_marker = Some(RespType::BulkString(size));
                    }
                }
                RespType::Integer(s) => lines.push(s),
                RespType::SimpleString(s) => lines.push(s),
                _ => return Err(MessageReaderError::UnknownDataType),
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
                        return Err(MessageReaderError::InvalidBulkStringFormat);
                    }
                    lines.push(line);
                    previous_marker = None;
                }
            }
            continue;
        }

        match RespType::from_str(line?.as_str())? {
            RespType::BulkString(size) => {
                if size == -1 {
                    return Err(MessageReaderError::UnknownDataType);
                } else {
                    previous_marker = Some(RespType::BulkString(size));
                }
            }
            RespType::Integer(s) => lines.push(s),
            RespType::SimpleString(s) => lines.push(s),
            _ => return Err(MessageReaderError::UnknownDataType),
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
    type Err = MessageReaderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        match chars.next() {
            Some('+') => Ok(RespType::SimpleString(chars.as_str().to_string())),
            Some('$') => {
                let size = chars
                    .as_str()
                    .parse::<i64>()
                    .map_err(|_| MessageReaderError::InvalidBulkStringFormat)?;
                Ok(RespType::BulkString(size))
            }
            Some('-') => Ok(RespType::Error),
            Some(':') => Ok(RespType::Integer(chars.as_str().to_string())),
            Some('*') => {
                let size = chars
                    .as_str()
                    .parse::<usize>()
                    .map_err(|_| MessageReaderError::InvalidArrayFormat)?;
                Ok(RespType::Array(size))
            }
            _ => Err(MessageReaderError::UnknownDataType),
        }
    }
}

#[derive(Debug, Error)]
pub enum MessageReaderError {
    #[error("connection error")]
    Io(#[from] std::io::Error),
    #[error("invalid RESP bulk string format")]
    InvalidBulkStringFormat,
    #[error("invalid RESP array format")]
    InvalidArrayFormat,
    #[error("unknown RESP data type")]
    UnknownDataType,
}
#[cfg(test)]
mod tests {
    use crate::redis::request_reader::read_message;
    use std::io;
    use std::string::String;

    #[test]
    fn test_read_integer() {
        assert_eq!(
            read_message(io::Cursor::new(b":1000\r\n")).unwrap(),
            vec!["1000".to_string()]
        );
    }

    #[test]
    fn test_read_simple_string() {
        assert_eq!(
            read_message(io::Cursor::new(b"+OK\r\n")).unwrap(),
            vec!["OK".to_string()]
        );
    }

    #[test]
    fn test_read_bulk_string() {
        assert_eq!(
            read_message(io::Cursor::new(b"$5\r\nhello\r\n")).unwrap(),
            vec!["hello".to_string()]
        );
    }

    #[test]
    fn test_read_empty_bulk_string() {
        assert_eq!(
            read_message(io::Cursor::new(b"$0\r\n\r\n")).unwrap(),
            vec!["".to_string()]
        );
    }

    #[test]
    fn test_read_array() {
        assert_eq!(
            read_message(io::Cursor::new(b"*2\r\n$4\r\nECHO\r\n$5\r\nmango\r\n")).unwrap(),
            vec!["ECHO".to_string(), "mango".to_string()]
        );
    }

    #[test]
    fn test_read_empty_stream() {
        let expected: Vec<String> = Vec::new();
        assert_eq!(read_message(io::Cursor::new(b"")).unwrap(), expected);
    }
}
