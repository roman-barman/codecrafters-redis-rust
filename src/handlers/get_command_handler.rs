use crate::commands::GetCommand;
use crate::handlers::CommandHandler;
use crate::storages::Storage;
use anyhow::Error;
use std::sync::{Arc, Mutex};

pub struct GetCommandHandler {
    storage: Arc<Mutex<dyn Storage>>,
}

impl GetCommandHandler {
    pub fn new(storage: Arc<Mutex<dyn Storage>>) -> Self {
        Self { storage }
    }
}

impl CommandHandler<GetCommand, Option<String>> for GetCommandHandler {
    fn handle(&self, command: &GetCommand) -> Result<Option<String>, Error> {
        match self.storage.lock()
            .map_err(|e| Error::msg(e.to_string()))?.
            get(command.as_ref())
            .map(String::from) {
            Some(s) => Ok(Some(s)),
            None => Ok(None),
        }
    }
}
