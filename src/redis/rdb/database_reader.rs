use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::PathBuf;
use thiserror::Error;

const RDB_MAGIC_STRING_SIZE: u8 = 5;
const RDB_VERSION_STRING_SIZE: u8 = 4;
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

fn is_rdb_file<T>(file: &mut T) -> Result<bool, DatabaseReaderError>
where
    T: Read + Seek,
{
    let mut magic_string = [0u8; RDB_MAGIC_STRING_SIZE as usize];
    file.seek(std::io::SeekFrom::Start(0))?;
    file.read_exact(&mut magic_string)?;
    Ok(&magic_string == b"REDIS")
}

fn read_rdb_version<T>(file: &mut T) -> Result<String, DatabaseReaderError>
where
    T: Read + Seek,
{
    let mut version = [0u8; RDB_VERSION_STRING_SIZE as usize];
    file.seek(std::io::SeekFrom::Start(RDB_MAGIC_STRING_SIZE as u64))?;
    file.read_exact(&mut version)?;
    Ok(std::str::from_utf8(&version)?.to_string())
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

fn read_metadata<T>(file: &mut T) -> Result<HashMap<String, String>, DatabaseReaderError>
where
    T: Read + Seek,
{
    let mut metadata = HashMap::new();
    let mut flag = [0u8; 1];
    file.read_exact(&mut flag)?;

    if flag[0] == AUX {
        loop {
            let key = read_string(file).map_err(|_| DatabaseReaderError::InvalidMetadataEncoding)?;
            let value = read_string(file).map_err(|_| DatabaseReaderError::InvalidMetadataEncoding)?;
            metadata.insert(key, value);

            let mut flag = [0u8; 1];
            file.read_exact(&mut flag)?;
            if flag[0] == SELECT_DB || flag[0] == EOF {
                break;
            }
            file.seek(std::io::SeekFrom::Current(-1))?;
        }
    }

    file.seek(std::io::SeekFrom::Current(-1))?;
    Ok(metadata)
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
    use crate::redis::rdb::database_reader::{is_rdb_file, read_length, read_metadata, read_rdb_version, read_string, AUX, SELECT_DB};
    use std::collections::HashMap;
    use std::io;

    #[test]
    fn test_is_rdb_file_true() {
        assert!(is_rdb_file(&mut io::Cursor::new(b"REDIS")).unwrap());
    }

    #[test]
    fn test_is_rdb_file_false() {
        assert!(!is_rdb_file(&mut io::Cursor::new(b"REDIT")).unwrap());
    }

    #[test]
    fn test_read_rdb_version() {
        assert_eq!(
            read_rdb_version(&mut io::Cursor::new(b"REDIS0001")).unwrap(),
            "0001".to_string()
        );
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
    fn test_read_metadata() {
        assert_eq!(
            read_metadata(&mut io::Cursor::new([
                AUX,
                0x4, 0x6b, 0x65, 0x79, 0x31,
                0x6, 0x76, 0x61, 0x6C, 0x75, 0x65, 0x31,
                0x4, 0x6b, 0x65, 0x79, 0x32,
                0x6, 0x76, 0x61, 0x6C, 0x75, 0x65, 0x32,
                SELECT_DB])).unwrap(),
            HashMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string())
            ])
        )
    }
}
