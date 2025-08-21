use std::collections::VecDeque;
use std::str::Chars;

const CR: char = '\r';
const LF: char = '\n';
const CRLF: &str = "\r\n";
const SIMPLE_STRING_PREFIX: char = '+';
const BULK_STRING_PREFIX: char = '$';
const ARRAY_PREFIX: char = '*';
const ERROR_PREFIX: char = '-';
const INTEGER_PREFIX: char = ':';

#[derive(Debug, PartialEq, Clone)]
pub enum RespType {
    SimpleString(String),
    BulkString(String),
    NullBulkString,
    Array(VecDeque<RespType>),
    Error(String),
    Integer(i64),
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum RespParseError {
    #[error("Empty RESP value")]
    EmptyValue,
    #[error("Unknown RESP type")]
    UnknownType,
    #[error("Invalid array length format")]
    InvalidArrayLengthFormat,
    #[error("Invalid bulk string length format")]
    InvalidBulkStringLengthFormat,
    #[error("Invalid integer format")]
    InvalidIntegerFormat,
    #[error("Unexpected end of input")]
    UnexpectedEof,
}

impl RespType {
    pub fn is_string(&self) -> bool {
        match self {
            RespType::SimpleString(_) => true,
            RespType::BulkString(_) => true,
            _ => false,
        }
    }

    pub fn get_string_value(self) -> Option<String> {
        match self {
            RespType::SimpleString(s) => Some(s),
            RespType::BulkString(s) => Some(s),
            _ => None,
        }
    }
}

impl TryFrom<&str> for RespType {
    type Error = RespParseError;
    fn try_from(value: &str) -> Result<Self, RespParseError> {
        let mut chars = value.chars();
        read_resp_type(&mut chars)
    }
}

fn read_resp_type(chars: &mut Chars) -> Result<RespType, RespParseError> {
    let first = chars.next();

    match first {
        Some(c) => match c {
            SIMPLE_STRING_PREFIX => read_simple_string(chars).map(RespType::SimpleString),
            BULK_STRING_PREFIX => read_bulk_string(chars).map(|x| match x {
                Some(s) => RespType::BulkString(s),
                None => RespType::NullBulkString,
            }),
            ERROR_PREFIX => read_error(chars).map(RespType::Error),
            ARRAY_PREFIX => read_array(chars).map(RespType::Array),
            INTEGER_PREFIX => read_integer(chars).map(RespType::Integer),
            _ => Err(RespParseError::UnknownType),
        },
        None => Err(RespParseError::EmptyValue),
    }
}

fn read_integer(chars: &mut Chars) -> Result<i64, RespParseError> {
    let value: String = chars.by_ref().take_while(|c| c != &CR).collect::<String>();
    if chars.next() != Some(LF) {
        return Err(RespParseError::UnexpectedEof);
    }

    value
        .parse()
        .map_err(|_| RespParseError::InvalidIntegerFormat)
}

fn read_array(chars: &mut Chars) -> Result<VecDeque<RespType>, RespParseError> {
    let len: String = chars.by_ref().take_while(|c| c != &CR).collect::<String>();
    if chars.next() != Some(LF) {
        return Err(RespParseError::UnexpectedEof);
    }

    let len: u64 = len
        .parse()
        .map_err(|_| RespParseError::InvalidArrayLengthFormat)?;
    let mut result = VecDeque::with_capacity(len as usize);

    for _ in 0..len {
        result.push_back(read_resp_type(chars)?);
    }

    Ok(result)
}

fn read_simple_string(chars: &mut Chars) -> Result<String, RespParseError> {
    let result = chars.take_while(|c| c != &CR).collect::<String>();
    if chars.next() != Some(LF) {
        Err(RespParseError::UnexpectedEof)
    } else {
        Ok(result)
    }
}

fn read_error(chars: &mut Chars) -> Result<String, RespParseError> {
    let result = chars.take_while(|c| c != &CR).collect::<String>();
    if chars.next() != Some(LF) {
        Err(RespParseError::UnexpectedEof)
    } else {
        Ok(result)
    }
}

fn read_bulk_string(chars: &mut Chars) -> Result<Option<String>, RespParseError> {
    let len: String = chars
        .by_ref()
        .take_while(|c| c != &'\r')
        .collect::<String>();
    if chars.next() != Some(LF) {
        return Err(RespParseError::UnexpectedEof);
    }

    let len: i64 = len
        .parse()
        .map_err(|_| RespParseError::InvalidBulkStringLengthFormat)?;
    let result = match len {
        ..0 => None,
        0 => Some("".to_string()),
        1.. => Some(chars.take(len as usize).collect()),
    };

    let next = chars.next();
    if next == None {
        return Ok(result);
    }
    if next != Some(CR) {
        return Err(RespParseError::UnexpectedEof);
    }
    if chars.next() != Some(LF) {
        return Err(RespParseError::UnexpectedEof);
    }

    Ok(result)
}

impl From<RespType> for String {
    fn from(resp_type: RespType) -> Self {
        match resp_type {
            RespType::SimpleString(s) => format!("{}{}{}", SIMPLE_STRING_PREFIX, s, CRLF),
            RespType::BulkString(s) => {
                format!("{}{}{}{}{}", BULK_STRING_PREFIX, s.len(), CRLF, s, CRLF)
            }
            RespType::NullBulkString => format!("{}-1{}", BULK_STRING_PREFIX, CRLF),
            RespType::Error(s) => format!("{}{}{}", ERROR_PREFIX, s, CRLF),
            RespType::Integer(i) => format!("{}{}{}", INTEGER_PREFIX, i, CRLF),
            RespType::Array(array) => {
                let mut result = format!("{}{}{}", ARRAY_PREFIX, array.len(), CRLF);
                for resp_type in array {
                    let resp_string: String = resp_type.into();
                    result.push_str(resp_string.as_str());
                }
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::resp_type::RespType;
    use std::collections::VecDeque;

    #[test]
    fn test_try_form_integer() {
        assert_eq!(RespType::try_from(":1000\r\n"), Ok(RespType::Integer(1000)));
    }

    #[test]
    fn test_try_form_simple_string() {
        assert_eq!(
            RespType::try_from("+OK\r\n"),
            Ok(RespType::SimpleString("OK".to_string()))
        );
    }

    #[test]
    fn test_try_form_bulk_string() {
        assert_eq!(
            RespType::try_from("$5\r\nhello\r\n"),
            Ok(RespType::BulkString("hello".to_string()))
        );
    }

    #[test]
    fn test_try_form_empty_bulk_string() {
        assert_eq!(
            RespType::try_from("$0\r\n\r\n"),
            Ok(RespType::BulkString("".to_string()))
        );
    }

    #[test]
    fn test_try_form_null_bulk_string() {
        assert_eq!(RespType::try_from("$-1\r\n"), Ok(RespType::NullBulkString));
    }

    #[test]
    fn test_try_form_error() {
        assert_eq!(
            RespType::try_from("-Error message\r\n"),
            Ok(RespType::Error("Error message".to_string()))
        );
    }

    #[test]
    fn test_try_form_array() {
        assert_eq!(
            RespType::try_from("*2\r\n$4\r\nECHO\r\n$5\r\nmango\r\n"),
            Ok(RespType::Array(VecDeque::from(vec![
                RespType::BulkString("ECHO".to_string()),
                RespType::BulkString("mango".to_string())
            ])))
        );
    }

    #[test]
    fn test_integer_to_string() {
        let result: String = RespType::Integer(1000).into();
        assert_eq!(result, ":1000\r\n".to_string());
    }

    #[test]
    fn test_simple_string_to_string() {
        let result: String = RespType::SimpleString("OK".to_string()).into();
        assert_eq!(result, "+OK\r\n".to_string());
    }

    #[test]
    fn test_bulk_string_to_string() {
        let result: String = RespType::BulkString("hello".to_string()).into();
        assert_eq!(result, "$5\r\nhello\r\n".to_string());
    }

    #[test]
    fn test_null_bulk_string_to_string() {
        let result: String = RespType::NullBulkString.into();
        assert_eq!(result, "$-1\r\n".to_string());
    }

    #[test]
    fn test_error_to_string() {
        let result: String = RespType::Error("Error message".to_string()).into();
        assert_eq!(result, "-Error message\r\n".to_string());
    }

    #[test]
    fn test_array_to_string() {
        let result: String = RespType::Array(VecDeque::from(vec![
            RespType::BulkString("hello".to_string()),
            RespType::BulkString("world".to_string()),
        ]))
        .into();
        assert_eq!(result, "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".to_string());
    }
}
