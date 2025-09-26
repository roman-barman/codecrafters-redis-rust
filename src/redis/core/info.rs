use crate::redis::core::request::Request;
use crate::redis::core::WriteResponse;
use crate::redis::Configuration;

pub fn info(
    writer: &mut impl WriteResponse,
    request: &Request,
    config: &Configuration,
) -> std::io::Result<()> {
    if request.len() > 2 {
        return writer.write_error("wrong number of arguments for 'info' command");
    }

    if request.len() == 2 {
        let section = request.get(1).unwrap();
        if section != "replication" {
            return writer.write_error(format!("unknown section: {}", section));
        }
    }

    if config.replicaof().is_some() {
        writer.write_bulk_sting(&Some("role:slave"))
    } else {
        writer.write_bulk_sting(&Some("role:master"))
    }
}
