use crate::redis::core::WriteResp;

pub fn ping(writer: &mut impl WriteResp) -> std::io::Result<()> {
    writer.write_simple_string("PONG")
}
