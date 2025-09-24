use crate::redis::rdb::constants::{AUX, EOF, EXPIRE_TIME, EXPIRE_TIME_MS, RESIZE_DB, SELECT_DB};
use crate::redis::rdb::ttl::Ttl;
use crc_fast::checksum_file;
use crc_fast::CrcAlgorithm::Crc64Redis;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;

type Database<'a> = Vec<(u32, Vec<(&'a str, (&'a str, &'a Ttl))>)>;

pub fn write_database(
    version: &str,
    metadata: Option<&Vec<(&str, &str)>>,
    databases: &Database,
    path: &Path,
    calculate_checksum: bool,
) -> Result<(), Error> {
    let mut file = File::create(path)?;

    file.write_all(b"REDIS")?;
    file.write_all(version.as_bytes().split_at(4).0)?;
    if let Some(metadata) = metadata {
        for (key, value) in metadata {
            file.write_all(&[AUX])?;
            write_string(&mut file, key)?;
            write_string(&mut file, value)?;
        }
    }
    for (number, data) in databases {
        file.write_all(&[SELECT_DB])?;
        write_length(&mut file, number)?;
        let db_size = data.len() as u32;
        let db_size_expire = data.iter().filter(|(_, (_, ttl))| ttl.is_expired()).count() as u32;
        file.write_all(&[RESIZE_DB])?;
        write_length(&mut file, &db_size)?;
        write_length(&mut file, &db_size_expire)?;

        for (key, (value, ttl)) in data {
            match ttl {
                Ttl::Seconds(seconds) => {
                    file.write_all(&[EXPIRE_TIME])?;
                    file.write_all(&seconds.to_le_bytes())?;
                    file.write_all(&[0])?;
                    write_string(&mut file, key)?;
                    write_string(&mut file, value)?;
                }
                Ttl::Milliseconds(milliseconds) => {
                    file.write_all(&[EXPIRE_TIME_MS])?;
                    file.write_all(&milliseconds.to_le_bytes())?;
                    file.write_all(&[0])?;
                    write_string(&mut file, key)?;
                    write_string(&mut file, value)?;
                }
                Ttl::None => {
                    file.write_all(&[0])?;
                    write_string(&mut file, key)?;
                    write_string(&mut file, value)?;
                }
            }
        }
    }

    file.write_all(&[EOF])?;

    if calculate_checksum {
        let checksum = checksum_file(Crc64Redis, path.to_str().unwrap(), None)?;
        file.write_all(checksum.to_be_bytes().as_slice())?;
    } else {
        file.write_all(0u64.to_be_bytes().as_slice())?;
    }

    Ok(())
}

fn write_length(writer: &mut File, length: &u32) -> Result<(), Error> {
    match length {
        0..64 => writer.write_all(&[*length as u8]),
        64..16384 => {
            let mut length_bytes = [0u8; 2];
            let bytes: [u8; 4] = length.to_be_bytes();
            length_bytes[0] = bytes[2] | 0b0100_0000;
            length_bytes[1] = bytes[3];
            writer.write_all(&length_bytes)
        }
        16384.. => {
            let mut length_bytes = [0u8; 5];
            let bytes: [u8; 4] = length.to_be_bytes();
            length_bytes[0] = 0b1000_0000;
            length_bytes[1] = bytes[0];
            length_bytes[2] = bytes[1];
            length_bytes[3] = bytes[2];
            length_bytes[4] = bytes[3];
            writer.write_all(&length_bytes)
        }
    }
}

fn write_string(writer: &mut File, string: &str) -> Result<(), Error> {
    write_length(writer, &(string.len() as u32))?;
    writer.write_all(string.as_bytes())
}
