use crate::resp::RespType;
use crate::storages::{KeySettingsBuilder, Storage};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const EXPIRY: &str = "px";

pub(crate) struct SetCommand {
    storage: Arc<Mutex<dyn Storage>>,
}

impl SetCommand {
    pub(crate) fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }

    pub(crate) fn execute(&mut self, args: &mut VecDeque<RespType>) -> RespType {
        if args.len() < 2 {
            return RespType::Error("SET requires at least 2 arguments.".to_string());
        }

        let key = args.pop_front().unwrap();
        let value = args.pop_front().unwrap();

        if !key.is_string() || !value.is_string() {
            return RespType::Error("SET requires string arguments for key and value.".to_string());
        }
        let mut key_settings_builder = KeySettingsBuilder::new();

        while !args.is_empty() {
            let arg_name = args.pop_front().unwrap();
            if !arg_name.is_string() {
                return RespType::Error("SET requires string arguments".to_string());
            }
            let arg_name = arg_name.get_string_value().unwrap().to_lowercase();
            match arg_name.as_str() {
                EXPIRY => {
                    if args.is_empty() {
                        return RespType::Error("No value specified for expiration.".to_string());
                    }
                    let value = args.pop_front().unwrap();
                    if value.is_string() {
                        let expiry = value.get_string_value().unwrap().parse::<i64>();
                        match expiry {
                            Ok(i) => {
                                if i < 1 {
                                    return RespType::Error("Expiration must be greater than 0.".to_string());
                                }
                                key_settings_builder = key_settings_builder.with_expiry(i as u64);
                            }
                            Err(_) => {
                                return RespType::Error("Invalid expiration value format.".to_string());
                            }
                        }
                    } else {
                        return RespType::Error("Expiration must be an string.".to_string());
                    }
                }
                _ => {
                    return RespType::Error(format!("Unknown '{}' argument for SET.", arg_name));
                }
            }
        }

        self.storage.lock().unwrap().set(
            key.get_string_value().unwrap().as_str(),
            value.get_string_value().unwrap().as_str(),
            key_settings_builder.build());
        RespType::SimpleString("OK".to_string())
    }
}
