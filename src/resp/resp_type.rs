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

fn read_error(chars: Chars) -> String {
    chars.take_while(|c| c != &'\r').collect::<String>()
}

#[cfg(test)]
mod tests {
    use crate::resp::resp_type::RespType;

    #[test]
    fn test_parse_simple_string() {
        assert_eq!(RespType::try_from("+OK\r\n"), Ok(RespType::SimpleString("OK".to_string())));
    }

    #[test]
    fn test_parse_bulk_string() {
        assert_eq!(RespType::try_from("$5\r\nhello\r\n"), Ok(RespType::BulkString("hello".to_string())));
    }

    #[test]
    fn test_parse_empty_bulk_string() {
        assert_eq!(RespType::try_from("$0\r\n\r\n"), Ok(RespType::BulkString("".to_string())));
    }

    #[test]
    fn test_parse_error() {
        assert_eq!(RespType::try_from("-Error message\r\n"), Ok(RespType::Error("Error message".to_string())));
    }
}
