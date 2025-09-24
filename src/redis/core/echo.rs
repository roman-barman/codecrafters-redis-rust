use crate::redis::core::request::Request;
use crate::redis::core::WriteResponse;

pub fn echo(writer: &mut impl WriteResponse, request: &Request) -> std::io::Result<()> {
    if request.len() != 2 {
        writer.write_error("wrong number of arguments for 'echo' command")
    } else {
        writer.write_bulk_sting(&request.get(1))
    }
}
