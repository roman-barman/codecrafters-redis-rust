use crate::redis::core::request::Request;
use crate::redis::core::WriteResp;

pub fn psync(writer: &mut impl WriteResp, request: &Request) -> std::io::Result<()> {
    if request.len() != 3 {
        return writer.write_error("wrong number of arguments for 'psync' command");
    }

    writer.write_simple_string("FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0")
}
