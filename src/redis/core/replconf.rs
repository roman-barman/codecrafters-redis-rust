use crate::redis::core::request::Request;
use crate::redis::core::WriteResp;

pub fn replconf(writer: &mut impl WriteResp, request: &Request) -> std::io::Result<()> {
    if request.len() != 3 {
        return writer.write_error("wrong number of arguments for 'replconf' command");
    }

    let config = request.get(1).unwrap().as_str();
    match config {
        "listening-port" => {
            let port = request.get(2).unwrap().parse::<u16>();
            match port {
                Ok(_) => writer.write_simple_string("OK"),
                Err(_) => writer.write_error("invalid port number"),
            }
        }
        "capa" => writer.write_simple_string("OK"),
        _ => writer.write_error(format!("unknown config: '{}'", config)),
    }
}
