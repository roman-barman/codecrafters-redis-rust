use crate::resp::RespType;
use crate::storages::Storage;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub(crate) struct GetCommand {
    storage: Arc<Mutex<dyn Storage>>,
}

impl GetCommand {
    pub(crate) fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }

    pub(crate) fn execute(&self, args: &mut VecDeque<RespType>) -> RespType {
        if args.len() != 1 {
            return RespType::Error("GET requires 1 arguments.".to_string());
        }

        let key = args.pop_front().unwrap();
        if !key.is_string() {
            return RespType::Error("GET requires string argument.".to_string());
        }
        let key = key.get_string_value().unwrap();
        match self.storage.lock().unwrap().get(key.as_str()).map(String::from) {
            Some(s) => RespType::BulkString(s),
            None => RespType::NullBulkString
        }
    }
}
