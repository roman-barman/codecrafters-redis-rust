use std::io::{Read, Seek};
use std::path::PathBuf;
use thiserror::Error;

const MAGIC_STRING_SIZE: u8 = 5;
const VERSION_STRING_SIZE: u8 = 4;
const EOF: u8 = 0xff;
const SELECT_DB: u8 = 0xfe;
const EXPIRE_TIME: u8 = 0xfd;
const EXPIRE_TIME_MS: u8 = 0xfc;
const RESIZE_DB: u8 = 0xfb;
const AUX: u8 = 0xfa;

pub struct DatabaseReader {
    path: PathBuf,
}

impl DatabaseReader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

fn read_length<T>(file: &mut T) -> Result<u32, DatabaseReaderError>
where
    T: Read,
{
    let mut length = [0u8; 1];
    file.read_exact(&mut length)?;

    let flag = length[0] >> 6;

    if flag == 0 {
        return Ok((length[0] & 0x3f) as u32);
    }

    if flag == 1 {
        let mut length_2 = [0u8; 1];
        file.read_exact(&mut length_2)?;
        return Ok(u32::from_be_bytes([0, 0, length[0] & 0x3f, length_2[0]]));
    }

    if flag == 2 {
        let mut length = [0u8; 4];
        file.read_exact(&mut length)?;
        return Ok(u32::from_be_bytes(length));
    }

    Err(DatabaseReaderError::InvalidLengthEncoding)
}

fn read_string<T>(file: &mut T) -> Result<String, DatabaseReaderError>
where
    T: Read,
{
    let length = read_length(file)?;
    let mut string = vec![0u8; length as usize];
    file.read_exact(&mut string)?;
    Ok(std::str::from_utf8(&string)?.to_string())
}

fn read_header_section<T>(file: &mut T) -> Result<(String, String), DatabaseReaderError>
where
    T: Read,
{
    let mut magic_string = [0u8; MAGIC_STRING_SIZE as usize];
    file.read_exact(&mut magic_string)?;
    let mut version = [0u8; VERSION_STRING_SIZE as usize];
    file.read_exact(&mut version)?;
    Ok((
        std::str::from_utf8(&magic_string)?.to_string(),
        std::str::from_utf8(&version)?.to_string(),
    ))
}

#[derive(Debug, Error)]
pub enum DatabaseReaderError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("invalid length encoding")]
    InvalidLengthEncoding,
    #[error("invalid metadata encoding")]
    InvalidMetadataEncoding,
}

#[cfg(test)]
mod tests {
    use crate::redis::rdb::database_reader::{read_header_section, read_length, read_string};
    use std::io;

    #[test]
    fn test_read_header_section() {
        assert_eq!(
            read_header_section(&mut io::Cursor::new(b"REDIS0001")).unwrap(),
            ("REDIS".to_string(), "0001".to_string())
        )
    }

    #[test]
    fn test_read_length_6_bits() {
        assert_eq!(read_length(&mut io::Cursor::new([0x2a])).unwrap(), 42);
    }

    #[test]
    fn test_read_length_14_bits() {
        assert_eq!(
            read_length(&mut io::Cursor::new([0x6a, 0xaa])).unwrap(),
            10922
        );
    }

    #[test]
    fn test_read_length_32_bits() {
        assert_eq!(
            read_length(&mut io::Cursor::new([0x80, 0xff, 0x00, 0xff, 0x00])).unwrap(),
            4278255360
        );
    }

    #[test]
    fn test_read_string() {
        assert_eq!(
            read_string(&mut io::Cursor::new([0x4, 0x74, 0x65, 0x73, 0x74])).unwrap(),
            "test".to_string()
        )
    }
}
