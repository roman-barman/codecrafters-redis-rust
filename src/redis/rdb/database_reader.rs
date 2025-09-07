use std::fs::File;
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

fn is_rdb_file(file: &mut File) -> Result<bool, DatabaseReaderError> {
    let mut magic_string = [0u8; RDB_MAGIC_STRING_SIZE as usize];
    file.seek(std::io::SeekFrom::Start(0))?;
    file.read_exact(&mut magic_string)?;
    Ok(&magic_string == b"REDIS")
}

fn read_rdb_version(file: &mut File) -> Result<String, DatabaseReaderError> {
    let mut version = [0u8; RDB_VERSION_STRING_SIZE as usize];
    file.seek(std::io::SeekFrom::Start(RDB_MAGIC_STRING_SIZE as u64))?;
    file.read_exact(&mut version)?;
    Ok(std::str::from_utf8(&version)?.to_string())
}

#[derive(Debug, Error)]
pub enum DatabaseReaderError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),
}
