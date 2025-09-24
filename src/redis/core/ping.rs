use crate::redis::core::WriteResponse;

pub fn ping(writer: &mut impl WriteResponse) -> std::io::Result<()> {
    writer.write_simple_string("PONG")
}
