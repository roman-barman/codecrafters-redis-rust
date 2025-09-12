use crate::redis::core::configuration::Configuration;
use crate::redis::core::echo::echo;
use crate::redis::core::error::Error;
use crate::redis::core::get_config::get_config;
use crate::redis::core::get_keys::get_keys;
use crate::redis::core::get_value::get_value;
use crate::redis::core::ping::ping;
use crate::redis::core::read_request::ReadRequest;
use crate::redis::core::request::Request;
use crate::redis::core::response::Response;
use crate::redis::core::set_key_value::set_key_value;
use crate::redis::core::storage::Storage;
use crate::redis::core::write_response::WriteResponse;

pub struct RequestHandler {
    storage: Box<dyn Storage>,
    configuration: Configuration,
}

impl RequestHandler {
    pub fn new(storage: Box<dyn Storage>, configuration: Configuration) -> Self {
        Self {
            storage,
            configuration,
        }
    }

    pub fn handle_request(
        &mut self,
        stream: &mut (impl ReadRequest + WriteResponse),
    ) -> Result<(), Error> {
        let request = stream
            .read_request()
            .map_err(|e| Error::Connection("cannot read request".to_string()))?;
        if request.len() == 0 {
            return Err(Error::Connection("empty request".to_string()));
        }
        let request = Request::new(request);
        log::info!("{:?}", request);
        let binding = request.get(0).unwrap().to_lowercase();
        let command = binding.as_str();
        let result = match command {
            "ping" => Ok(ping()),
            "echo" => echo(&request).map_err(|e| e.into()),
            "get" => get_value(&mut self.storage, &request).map_err(|e| e.into()),
            "set" => set_key_value(&mut self.storage, &request).map_err(|e| e.into()),
            "config" => get_config(&request, &self.configuration).map_err(|e| e.into()),
            "keys" => Ok(get_keys(&mut self.storage)),
            _ => Err(Error::Client(format!("Unknown command '{}'", command))),
        };

        log::info!("{:?}", result);
        let write_result = match result {
            Ok(response) => match response {
                Response::SimpleString(value) => stream.write_simple_string(value),
                Response::BulkString(value) => stream.write_bulk_sting(&value),
                Response::Array(value) => stream.write_array(&value),
            },
            Err(e) => match e {
                Error::Client(e) => {
                    log::info!("{}", e);
                    stream.write_error(&e)
                }
                _ => Err(e)?,
            },
        };

        write_result.map_err(|_| Error::Connection("cannot write response".to_string()))
    }
}
