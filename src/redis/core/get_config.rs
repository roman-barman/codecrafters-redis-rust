use crate::redis::core::configuration::Configuration;
use crate::redis::core::request::Request;
use crate::redis::core::WriteResponse;

const DIR: &str = "dir";
const DB_FILE_NAME: &str = "dbfilename";

pub fn get_config(
    writer: &mut impl WriteResponse,
    request: &Request,
    config: &Configuration,
) -> std::io::Result<()> {
    if request.len() != 3 {
        return writer.write_error("wrong number of arguments for 'config' command");
    }

    let arg = request.get(1).unwrap();
    if arg.to_lowercase() != "get" {
        return writer.write_error(format!("unknown argument: '{}'", arg));
    }

    let parameter = request.get(2).unwrap();

    if parameter.eq_ignore_ascii_case(DIR) {
        writer.write_array(&[Some(DIR), config.dir().map(|x| x.as_str())])
    } else if parameter.eq_ignore_ascii_case(DB_FILE_NAME) {
        writer.write_array(&[
            Some(DB_FILE_NAME),
            config.db_file_name().map(|x| x.as_str()),
        ])
    } else {
        writer.write_error("unknown configuration parameter")
    }
}
