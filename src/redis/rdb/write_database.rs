use crate::redis::rdb::constants::{AUX, EOF, EXPIRE_TIME, EXPIRE_TIME_MS, RESIZE_DB, SELECT_DB};
use crate::redis::rdb::ttl::Ttl;
use std::io;
use std::io::{Cursor, Error, Write};

pub fn write_database<T>(
    version: &str,
    metadata: Option<&Vec<(&str, &str)>>,
    databases: &Vec<(u32, Vec<(&str, (&str, &Ttl))>)>,
    writer: &mut T,
    calculate_checksum: bool,
) -> Result<(), Error>
where
    T: Write,
{
    let mut cursor = Cursor::new(Vec::<u8>::new());

    cursor.write_all(b"REDIS")?;
    cursor.write_all(version.as_bytes().split_at(4).0)?;
    if let Some(metadata) = metadata {
        for (key, value) in metadata {
            cursor.write_all(&[AUX])?;
            write_string(&mut cursor, key)?;
            write_string(&mut cursor, value)?;
        }
    }
    for (number, data) in databases {
        cursor.write_all(&[SELECT_DB])?;
        write_length(&mut cursor, number)?;
        let db_size = data.len() as u32;
        let db_size_expire = data.iter().filter(|(_, (_, ttl))| ttl.is_expired()).count() as u32;
        cursor.write_all(&[RESIZE_DB])?;
        write_length(&mut cursor, &db_size)?;
        write_length(&mut cursor, &db_size_expire)?;

        for (key, (value, ttl)) in data {
            match ttl {
                Ttl::Seconds(seconds) => {
                    cursor.write_all(&[EXPIRE_TIME])?;
                    cursor.write_all(&seconds.to_be_bytes())?;
                    cursor.write_all(&[0])?;
                    write_string(&mut cursor, key)?;
                    write_string(&mut cursor, value)?;
                }
                Ttl::Milliseconds(milliseconds) => {
                    cursor.write_all(&[EXPIRE_TIME_MS])?;
                    cursor.write_all(&milliseconds.to_be_bytes())?;
                    cursor.write_all(&[0])?;
                    write_string(&mut cursor, key)?;
                    write_string(&mut cursor, value)?;
                }
                Ttl::None => {
                    cursor.write_all(&[0])?;
                    write_string(&mut cursor, key)?;
                    write_string(&mut cursor, value)?;
                }
            }
        }
    }

    cursor.write_all(&[EOF])?;

    if calculate_checksum {
        let checksum = crc64::crc64(0, cursor.get_ref().as_slice());
        cursor.write_all(checksum.to_be_bytes().as_slice())?;
    } else {
        cursor.write_all(0u64.to_be_bytes().as_slice())?;
    }

    io::copy(&mut cursor.get_ref().as_slice(), writer)?;

    Ok(())
}

fn write_length<T>(writer: &mut T, length: &u32) -> Result<(), Error>
where
    T: Write,
{
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

fn write_string<T>(writer: &mut T, string: &str) -> Result<(), Error>
where
    T: Write,
{
    write_length(writer, &(string.len() as u32))?;
    writer.write_all(string.as_bytes())
}
