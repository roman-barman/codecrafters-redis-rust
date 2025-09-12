use crate::redis::core::response::Response;
use crate::redis::core::Storage;

pub fn get_keys(storage: &mut Box<dyn Storage>) -> Response {
    Response::Array(
        storage
            .get_keys()
            .into_iter()
            .map(|x| Some(x.to_string()))
            .collect(),
    )
}
