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

/*fn read_length(file: &mut File) -> Result<u64, DatabaseReaderError> {
    let mut length = [0u8; 1];
    file.read_exact(&mut length)?;

    let flag = length[0] & 0xc0;

    if flag == 0 {
        return Ok((length[0] & 0x3f) as u64);
    }

    if flag == 0x40 {
        let mut length = [0u8; 1];
        file.read_exact(&mut length)?;
        return Ok(((length[0] as u64) << 8) | (length[1] as u64));
    }
}*/

#[derive(Debug, Error)]
pub enum DatabaseReaderError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),
}

#[cfg(test)]
mod tests {
    use crate::redis::rdb::database_reader::is_rdb_file;
    use std::io;

    #[test]
    fn test_is_rdb_file_true() {
        assert!(is_rdb_file(&mut io::Cursor::new(b"REDIS")).unwrap());
    }

    #[test]
    fn test_is_rdb_file_false() {
        assert!(!is_rdb_file(&mut io::Cursor::new(b"REDIT")).unwrap());
    }
}
