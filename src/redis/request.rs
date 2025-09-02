use thiserror::Error;

pub struct Request {
    value: Vec<String>,
}

impl Request {
    pub fn new(value: Vec<Option<String>>) -> Result<Self, RequestError> {
        if value.is_empty() {
            return Err(RequestError::EmptyRequest);
        }
        if value.iter().any(|x| x.is_none()) {
            return Err(RequestError::InvalidRequest);
        }
        Ok(Self {
            value: value.into_iter().map(|x| x.unwrap()).collect(),
        })
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.value.get(index)
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("request is empty")]
    EmptyRequest,
    #[error("invalid request")]
    InvalidRequest,
}
