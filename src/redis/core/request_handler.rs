use crate::redis::core::configuration::Configuration;
use crate::redis::core::echo::echo;
use crate::redis::core::get_config::get_config;
use crate::redis::core::get_keys::get_keys;
use crate::redis::core::get_value::get_value;
use crate::redis::core::info::info;
use crate::redis::core::ping::ping;
use crate::redis::core::read_request::ReadRequest;
use crate::redis::core::request::Request;
use crate::redis::core::save::save;
use crate::redis::core::set_key_value::set_key_value;
use crate::redis::core::write_response::WriteResponse;
use crate::redis::rdb::RedisStorage;
use std::fmt::Display;
use std::rc::Rc;

pub struct RequestHandler {
    storage: RedisStorage,
    configuration: Rc<Configuration>,
}

impl RequestHandler {
    pub fn new(storage: RedisStorage, configuration: Rc<Configuration>) -> Self {
        Self {
            storage,
            configuration,
        }
    }

    pub fn handle_request(
        &mut self,
        stream: &mut (impl ReadRequest + WriteResponse),
    ) -> Result<(), Error> {
        let request = stream.read_request();
        let request = match request {
            Ok(request) => {
                if request.is_empty() {
                    return Err(Error {
                        msg: "empty request".to_string(),
                    });
                }
                Request::new(request)
            }
            Err(_) => {
                return Err(Error {
                    msg: "can not read request".to_string(),
                })
            }
        };
        log::info!("{:?}", request);
        let binding = request.get(0).unwrap().to_lowercase();
        let command = binding.as_str();
        let result = match command {
            "ping" => ping(stream),
            "echo" => echo(stream, &request),
            "get" => get_value(stream, &mut self.storage, &request),
            "set" => set_key_value(stream, &mut self.storage, &request),
            "config" => get_config(stream, &request, &self.configuration),
            "keys" => get_keys(stream, &mut self.storage),
            "save" => save(stream, &mut self.storage, &self.configuration),
            "info" => info(stream, &request),
            _ => stream.write_error(format!("Unknown command '{}'", command)),
        };

        result.map_err(|_| Error {
            msg: "cannot write response".to_string(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub struct Error {
    msg: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
