use crate::resp::RespType;
use crate::storages::Storage;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub(crate) struct SetCommand {
    storage: Arc<Mutex<dyn Storage>>,
}

impl SetCommand {
    pub(crate) fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }

    pub(crate) fn execute(&mut self, args: &mut VecDeque<RespType>) -> RespType {
        if args.len() != 2 {
            return RespType::Error("SET requires 2 arguments.".to_string());
        }

        let key = args.pop_front().unwrap();
        let value = args.pop_front().unwrap();

        if !key.is_string() || !value.is_string() {
            return RespType::Error("SET requires string arguments.".to_string());
        }

        self.storage.lock().unwrap()
            .set(key.get_string_value().unwrap().as_str(), value.get_string_value().unwrap().as_str());
        RespType::SimpleString("OK".to_string())
    }
}
