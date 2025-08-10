use crate::config::Config;
use crate::resp::RespType;
use std::collections::VecDeque;
use std::sync::Arc;

const CONFIG_GET: &str = "CONFIG GET";
const DIR: &str = "dir";
const DB_FILE_NAME: &str = "dbfilename";

pub(crate) struct GetConfigCommand {
    config: Arc<Config>,
}

impl GetConfigCommand {
    pub(crate) fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub(crate) fn execute(&self, args: &mut VecDeque<RespType>) -> RespType {
        if args.is_empty() {
            return RespType::Error(format!("{} requires arguments", CONFIG_GET));
        }

        let arg = args.pop_front().unwrap();
        if arg.is_string() {
            let arg = arg.get_string_value().unwrap();
            match arg.as_str() {
                DIR => RespType::Array(VecDeque::from(vec![
                    RespType::BulkString(DIR.to_string()),
                    RespType::BulkString(self.config.dir.as_ref().map_or("".to_string(), |x| x.clone()))
                ])),
                DB_FILE_NAME => RespType::Array(VecDeque::from(vec![
                    RespType::BulkString(DIR.to_string()),
                    RespType::BulkString(self.config.dbfilename.as_ref().map_or("".to_string(), |x| x.clone()))
                ])),
                _ => RespType::Error(format!("{} unknown argument", arg)),
            }
        } else {
            return RespType::Error(format!("{} requires string argument", CONFIG_GET));
        }
    }
}
