use crate::commands::Command;

pub struct SetCommand {
    key: String,
    value: String,
    expiry: Option<u64>,
}

impl SetCommand {
    pub fn new(key: String, value: String, expiry: Option<u64>) -> Self {
        Self {
            key,
            value,
            expiry,
        }
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn get_expiry(&self) -> &Option<u64> {
        &self.expiry
    }
}

impl Command<String> for SetCommand {}
