use std::collections::HashMap;
use std::io::Read;
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

fn read_section<T>(file: &mut T) -> Result<Section, DatabaseReaderError>
where
    T: Read,
{
    let mut op_code = [0u8; 1];
    file.read_exact(&mut op_code)?;
    match op_code[0] {
        AUX => {
            let key = read_string(file)?;
            let value = read_string(file)?;
            Ok(Section::Metadata(key, value))
        }
        SELECT_DB => Ok(Section::Database(
            read_length(file)?,
            read_database_section(file)?,
        )),
        EOF => {
            let mut checksum = [0u8; 8];
            file.read_exact(&mut checksum)?;
            Ok(Section::Checksum(u64::from_be_bytes(checksum)))
        }
        _ => Err(DatabaseReaderError::InvalidFileEncoding),
    }
}

fn read_database_section<T>(
    file: &mut T,
) -> Result<HashMap<String, (String, Option<u64>)>, DatabaseReaderError>
where
    T: Read,
{
    let mut fb = [0u8; 1];
    file.read_exact(&mut fb)?;

    if fb[0] != RESIZE_DB {
        return Err(DatabaseReaderError::InvalidFileEncoding);
    }

    let db_size = read_length(file)?;
    let db_size_expire = read_length(file)?;
    let mut db = HashMap::with_capacity(db_size as usize);

    let mut current_db_size = 0;
    let mut current_db_size_expire = 0;

    while current_db_size < db_size {
        let mut byte = [0u8; 1];
        file.read_exact(&mut byte)?;

        match byte[0] {
            EXPIRE_TIME => {
                let mut expire_time = [0u8; 4];
                file.read_exact(&mut expire_time)?;
                let expire_time = u32::from_be_bytes(expire_time);
                read_value_type(file)?;
                current_db_size_expire += 1;
                current_db_size += 1;

                if current_db_size_expire > db_size_expire {
                    return Err(DatabaseReaderError::InvalidFileEncoding);
                }

                db.insert(
                    read_string(file)?,
                    (read_string(file)?, Some(expire_time as u64 * 1000)),
                );
            }
            EXPIRE_TIME_MS => {
                let mut expire_time = [0u8; 8];
                file.read_exact(&mut expire_time)?;
                let expire_time = u64::from_be_bytes(expire_time);
                read_value_type(file)?;
                current_db_size_expire += 1;
                current_db_size += 1;

                if current_db_size_expire > db_size_expire {
                    return Err(DatabaseReaderError::InvalidFileEncoding);
                }

                db.insert(read_string(file)?, (read_string(file)?, Some(expire_time)));
            }
            _ => {
                if !is_supported_value_type(byte[0]) {
                    return Err(DatabaseReaderError::UnsupportedValueType);
                }
                current_db_size += 1;
                db.insert(read_string(file)?, (read_string(file)?, None));
            }
        }
    }

    Ok(db)
}

fn read_value_type<T>(file: &mut T) -> Result<(), DatabaseReaderError>
where
    T: Read,
{
    let mut byte = [0u8; 1];
    file.read_exact(&mut byte)?;
    if !is_supported_value_type(byte[0]) {
        return Err(DatabaseReaderError::UnsupportedValueType);
    }
    Ok(())
}

fn is_supported_value_type(byte: u8) -> bool {
    byte == 0
}

#[derive(Debug, PartialEq)]
enum Section {
    Metadata(String, String),
    Database(u32, HashMap<String, (String, Option<u64>)>),
    Checksum(u64),
}

#[derive(Debug, Error)]
pub enum DatabaseReaderError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("invalid length encoding")]
    InvalidLengthEncoding,
    #[error("invalid file encoding")]
    InvalidFileEncoding,
    #[error("unsupported value type")]
    UnsupportedValueType,
}

#[cfg(test)]
mod tests {
    use crate::redis::rdb::database_reader::{
        read_header_section, read_length, read_section, read_string, Section, AUX, EOF,
    };
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

    #[test]
    fn test_read_metadata_section() {
        assert_eq!(
            read_section(&mut io::Cursor::new([
                AUX, 0x3, 0x6B, 0x65, 0x79, 0x5, 0x76, 0x61, 0x6C, 0x75, 0x65
            ]))
                .unwrap(),
            Section::Metadata("key".to_string(), "value".to_string())
        )
    }

    #[test]
    fn test_read_checksum_section() {
        assert_eq!(
            read_section(&mut io::Cursor::new([
                EOF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa
            ]))
                .unwrap(),
            Section::Checksum(10)
        )
    }
}
