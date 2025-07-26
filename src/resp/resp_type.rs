use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum RespType {
    SimpleString(String),
    BulkString(String),
    Error(String),
}

impl TryFrom<&str> for RespType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, String> {
        let mut chars = value.chars();
        let first = chars.next();

        match first {
            Some(c) => {
                match c {
                    '+' => Ok(RespType::SimpleString(read_simple_string(chars))),
                    '$' => read_bulk_string(&mut chars).map(RespType::BulkString),
                    '-' => Ok(RespType::Error(read_error(chars))),
                    _ => Err("Unknown RESP type".to_string())
                }
            }
            None => {
                Err("Empty command".to_string())
            }
        }
    }
}

fn read_simple_string(chars: Chars) -> String {
    chars.take_while(|c| c != &'\r').collect::<String>()
}

fn read_bulk_string(chars: &mut Chars) -> Result<String, String> {
    let len: String = chars.by_ref().take_while(|c| c != &'\r').collect::<String>();

    chars.next();

    let len: u64 = len.parse().map_err(|_| "Invalid bulk string length".to_string())?;

    if len < 1 {
        return Ok("".to_string());
    }

    let content: String = chars.take(len as usize).collect();
    Ok(content)
}

impl From<RespType> for String {
    fn from(resp_type: RespType) -> Self {
        match resp_type {
            RespType::SimpleString(s) => format!("+{}\r\n", s),
            RespType::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            RespType::Error(s) => format!("-{}\r\n", s),
        }
    }
}

fn read_error(chars: Chars) -> String {
    chars.take_while(|c| c != &'\r').collect::<String>()
}

#[cfg(test)]
mod tests {
    use crate::resp::resp_type::RespType;

    #[test]
    fn test_try_form_simple_string() {
        assert_eq!(RespType::try_from("+OK\r\n"), Ok(RespType::SimpleString("OK".to_string())));
    }

    #[test]
    fn test_try_form_bulk_string() {
        assert_eq!(RespType::try_from("$5\r\nhello\r\n"), Ok(RespType::BulkString("hello".to_string())));
    }

    #[test]
    fn test_try_form_empty_bulk_string() {
        assert_eq!(RespType::try_from("$0\r\n\r\n"), Ok(RespType::BulkString("".to_string())));
    }

    #[test]
    fn test_try_form_error() {
        assert_eq!(RespType::try_from("-Error message\r\n"), Ok(RespType::Error("Error message".to_string())));
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
    fn test_error_to_string() {
        let result: String = RespType::Error("Error message".to_string()).into();
        assert_eq!(result, "-Error message\r\n".to_string());
    }
}
