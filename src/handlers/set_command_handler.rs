use crate::commands::{Command, SetCommand};
use crate::handlers::CommandHandler;
use crate::storages::{KeySettingsBuilder, Storage};
use anyhow::Error;
use std::sync::{Arc, Mutex};

pub struct SetCommandHandler {
    storage: Arc<Mutex<dyn Storage>>,
}

impl SetCommandHandler {
    pub fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }
}

impl CommandHandler<SetCommand, String> for SetCommandHandler {
    fn handle(&self, command: &SetCommand) -> Result<String, Error>
    where
        SetCommand: Command<String>,
    {
        let mut key_settings_builder = KeySettingsBuilder::new();
        key_settings_builder = match command.get_expiry() {
            Some(value) => key_settings_builder.with_expiry(*value),
            None => key_settings_builder
        };

        self.storage.lock().map_err(|e| Error::msg(e.to_string()))?
            .set(command.get_key(), command.get_value(), key_settings_builder.build());
        Ok("OK".to_string())
    }
}
