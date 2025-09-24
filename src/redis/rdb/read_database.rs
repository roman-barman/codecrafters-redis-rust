use crate::redis::rdb::constants::{AUX, EOF, EXPIRE_TIME, EXPIRE_TIME_MS, RESIZE_DB, SELECT_DB};
use crate::redis::rdb::ttl::Ttl;
use crc_fast::{CrcAlgorithm, Digest};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

const MAGIC_STRING_SIZE: u8 = 5;
const VERSION_STRING_SIZE: u8 = 4;

type ReadResult = Result<Option<HashMap<String, (String, Ttl)>>, DatabaseReaderError>;

pub fn read_databases(path: &Path) -> ReadResult {
    let mut file = File::open(path)?;
    let mut digest = Digest::new(CrcAlgorithm::Crc64Redis);
    let mut digest_option = Some(&mut digest);

    log::debug!("reading first database");
    let (magic_string, version) = read_header_section(&mut file, &mut digest_option)?;
    log::debug!("version: {}", version);
    log::debug!("magic string: {}", magic_string);
    if magic_string != "REDIS" {
        return Err(DatabaseReaderError::UnsupportedFileFormat);
    }

    let mut databases = vec![];

    loop {
        let section = read_section(&mut file, &mut digest_option)?;
        match section {
            Section::Metadata(key, value) => {
                log::debug!("metadata: {}: {}", key, value);
            }
            Section::Database(_, data) => databases.push(data),
            Section::Checksum(checksum) => {
                log::debug!("checksum: {}", checksum);
                let calculated_checksum = digest.finalize();
                log::debug!("calculated checksum: {}", calculated_checksum);
                break;
            }
        }
    }

    Ok(databases.pop())
}
fn read_length<T>(
    file: &mut T,
    digest: &mut Option<&mut Digest>,
) -> Result<LengthEncoding, DatabaseReaderError>
where
    T: Read,
{
    log::debug!("reading length");
    let mut length = [0u8; 1];
    file.read_exact(&mut length)?;
    copy_to_digest(digest, &length);

    let flag = length[0] >> 6;
    log::debug!("length flag: {}", flag);

    if flag == 0 {
        return Ok(LengthEncoding::Bits6(length[0] & 0x3f));
    }

    if flag == 1 {
        let mut length_2 = [0u8; 1];
        file.read_exact(&mut length_2)?;
        copy_to_digest(digest, &length_2);
        return Ok(LengthEncoding::Bits14(u16::from_be_bytes([
            length[0] & 0x3f,
            length_2[0],
        ])));
    }

    if flag == 2 {
        let mut length = [0u8; 4];
        file.read_exact(&mut length)?;
        copy_to_digest(digest, &length);
        return Ok(LengthEncoding::Bits32(u32::from_be_bytes(length)));
    }

    Ok(LengthEncoding::Special(length[0] & 0x3f))
}

fn read_string<T>(
    file: &mut T,
    digest: &mut Option<&mut Digest>,
) -> Result<String, DatabaseReaderError>
where
    T: Read,
{
    log::debug!("reading string");
    let length = read_length(file, digest)?;
    if let LengthEncoding::Special(flag) = length {
        log::debug!("special length: {}", flag);
        match flag {
            0 => {
                let mut value = [0u8; 1];
                file.read_exact(&mut value)?;
                copy_to_digest(digest, &value);
                Ok(value[0].to_string())
            }
            1 => {
                let mut value = [0u8; 2];
                file.read_exact(&mut value)?;
                copy_to_digest(digest, &value);
                Ok(u16::from_be_bytes(value).to_string())
            }
            2 => {
                let mut value = [0u8; 4];
                file.read_exact(&mut value)?;
                copy_to_digest(digest, &value);
                Ok(u32::from_be_bytes(value).to_string())
            }
            _ => Err(DatabaseReaderError::InvalidFileEncoding),
        }
    } else {
        let mut string = vec![0u8; length.get_length()? as usize];
        file.read_exact(&mut string)?;
        copy_to_digest(digest, &string);
        Ok(std::str::from_utf8(&string)?.to_string())
    }
}

fn read_header_section<T>(
    file: &mut T,
    digest: &mut Option<&mut Digest>,
) -> Result<(String, String), DatabaseReaderError>
where
    T: Read,
{
    log::debug!("reading header section");
    let mut magic_string = [0u8; MAGIC_STRING_SIZE as usize];
    file.read_exact(&mut magic_string)?;
    copy_to_digest(digest, &magic_string);
    let mut version = [0u8; VERSION_STRING_SIZE as usize];
    file.read_exact(&mut version)?;
    copy_to_digest(digest, &version);
    Ok((
        std::str::from_utf8(&magic_string)?.to_string(),
        std::str::from_utf8(&version)?.to_string(),
    ))
}

fn read_section<T>(
    file: &mut T,
    digest: &mut Option<&mut Digest>,
) -> Result<Section, DatabaseReaderError>
where
    T: Read,
{
    log::debug!("reading section");
    let mut op_code = [0u8; 1];
    file.read_exact(&mut op_code)?;
    copy_to_digest(digest, &op_code);
    log::debug!("op code: {:x}", op_code[0]);
    match op_code[0] {
        AUX => {
            let key = read_string(file, digest)?;
            let value = read_string(file, digest)?;
            Ok(Section::Metadata(key, value))
        }
        SELECT_DB => Ok(Section::Database(
            read_length(file, digest)?.get_length()?,
            read_database_section(file, digest)?,
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
    digest: &mut Option<&mut Digest>,
) -> Result<HashMap<String, (String, Ttl)>, DatabaseReaderError>
where
    T: Read,
{
    let mut fb = [0u8; 1];
    file.read_exact(&mut fb)?;
    copy_to_digest(digest, &fb);

    if fb[0] != RESIZE_DB {
        return Err(DatabaseReaderError::InvalidFileEncoding);
    }

    let db_size = read_length(file, digest)?.get_length()?;
    let db_size_expire = read_length(file, digest)?.get_length()?;
    let mut db = HashMap::with_capacity(db_size as usize);

    let mut current_db_size = 0;
    let mut current_db_size_expire = 0;

    while current_db_size < db_size {
        let mut byte = [0u8; 1];
        file.read_exact(&mut byte)?;
        copy_to_digest(digest, &byte);

        match byte[0] {
            EXPIRE_TIME => {
                let mut expire_time = [0u8; 4];
                file.read_exact(&mut expire_time)?;
                copy_to_digest(digest, &expire_time);
                let expire_time = u32::from_le_bytes(expire_time);
                read_value_type(file, digest)?;
                current_db_size_expire += 1;
                current_db_size += 1;

                if current_db_size_expire > db_size_expire {
                    return Err(DatabaseReaderError::InvalidFileEncoding);
                }

                db.insert(
                    read_string(file, digest)?,
                    (read_string(file, digest)?, Ttl::Seconds(expire_time)),
                );
            }
            EXPIRE_TIME_MS => {
                let mut expire_time = [0u8; 8];
                file.read_exact(&mut expire_time)?;
                copy_to_digest(digest, &expire_time);
                let expire_time = u64::from_le_bytes(expire_time);
                read_value_type(file, digest)?;
                current_db_size_expire += 1;
                current_db_size += 1;

                if current_db_size_expire > db_size_expire {
                    return Err(DatabaseReaderError::InvalidFileEncoding);
                }

                db.insert(
                    read_string(file, digest)?,
                    (read_string(file, digest)?, Ttl::Milliseconds(expire_time)),
                );
            }
            _ => {
                if !is_supported_value_type(byte[0]) {
                    return Err(DatabaseReaderError::UnsupportedValueType);
                }
                current_db_size += 1;
                db.insert(
                    read_string(file, digest)?,
                    (read_string(file, digest)?, Ttl::None),
                );
            }
        }
    }

    Ok(db)
}

fn read_value_type<T>(
    file: &mut T,
    digest: &mut Option<&mut Digest>,
) -> Result<(), DatabaseReaderError>
where
    T: Read,
{
    let mut byte = [0u8; 1];
    file.read_exact(&mut byte)?;
    copy_to_digest(digest, &byte);
    if !is_supported_value_type(byte[0]) {
        return Err(DatabaseReaderError::UnsupportedValueType);
    }
    Ok(())
}

fn is_supported_value_type(byte: u8) -> bool {
    byte == 0
}

fn copy_to_digest(digest: &mut Option<&mut Digest>, data: &[u8]) {
    if let Some(digest) = digest {
        digest.update(data);
    }
}

#[derive(Debug, PartialEq)]
enum Section {
    Metadata(String, String),
    Database(u32, HashMap<String, (String, Ttl)>),
    Checksum(u64),
}

#[derive(Debug, PartialEq)]
enum LengthEncoding {
    Bits6(u8),
    Bits14(u16),
    Bits32(u32),
    Special(u8),
}

impl LengthEncoding {
    pub fn get_length(&self) -> Result<u32, DatabaseReaderError> {
        match self {
            LengthEncoding::Bits6(length) => Ok(*length as u32),
            LengthEncoding::Bits14(length) => Ok(*length as u32),
            LengthEncoding::Bits32(length) => Ok(*length),
            LengthEncoding::Special(_) => Err(DatabaseReaderError::InvalidFileEncoding),
        }
    }
}

#[derive(Debug, Error)]
pub enum DatabaseReaderError {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("UTF8 error")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("invalid file encoding")]
    InvalidFileEncoding,
    #[error("unsupported value type")]
    UnsupportedValueType,
    #[error("unsupported file format")]
    UnsupportedFileFormat,
}

#[cfg(test)]
mod tests {
    use crate::redis::rdb::read_database::{
        read_header_section, read_length, read_section, read_string, LengthEncoding, Section, AUX,
        EOF,
    };
    use std::io;

    #[test]
    fn test_read_header_section() {
        assert_eq!(
            read_header_section(&mut io::Cursor::new(b"REDIS0001"), &mut None).unwrap(),
            ("REDIS".to_string(), "0001".to_string())
        )
    }

    #[test]
    fn test_read_length_6_bits() {
        assert_eq!(
            read_length(&mut io::Cursor::new([0x2a]), &mut None).unwrap(),
            LengthEncoding::Bits6(42)
        );
    }

    #[test]
    fn test_read_length_14_bits() {
        assert_eq!(
            read_length(&mut io::Cursor::new([0x6a, 0xaa]), &mut None).unwrap(),
            LengthEncoding::Bits14(10922)
        );
    }

    #[test]
    fn test_read_length_32_bits() {
        assert_eq!(
            read_length(
                &mut io::Cursor::new([0x80, 0xff, 0x00, 0xff, 0x00]),
                &mut None
            )
            .unwrap(),
            LengthEncoding::Bits32(4278255360)
        );
    }

    #[test]
    fn test_read_string() {
        assert_eq!(
            read_string(
                &mut io::Cursor::new([0x4, 0x74, 0x65, 0x73, 0x74]),
                &mut None
            )
            .unwrap(),
            "test".to_string()
        )
    }

    #[test]
    fn test_read_metadata_section() {
        assert_eq!(
            read_section(
                &mut io::Cursor::new([
                    AUX, 0x3, 0x6B, 0x65, 0x79, 0x5, 0x76, 0x61, 0x6C, 0x75, 0x65
                ]),
                &mut None
            )
            .unwrap(),
            Section::Metadata("key".to_string(), "value".to_string())
        )
    }

    #[test]
    fn test_read_checksum_section() {
        assert_eq!(
            read_section(
                &mut io::Cursor::new([EOF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xa]),
                &mut None
            )
            .unwrap(),
            Section::Checksum(10)
        )
    }
}
