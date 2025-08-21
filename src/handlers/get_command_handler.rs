use crate::commands::GetCommand;
use crate::handlers::CommandHandler;
use crate::storages::Storage;
use anyhow::Error;
use std::cell::RefCell;
use std::rc::Rc;

pub struct GetCommandHandler {
    storage: Rc<RefCell<dyn Storage>>,
}

impl GetCommandHandler {
    pub fn new(storage: Rc<RefCell<dyn Storage>>) -> Self {
        Self { storage }
    }
}

impl CommandHandler<GetCommand, Option<String>> for GetCommandHandler {
    fn handle(&mut self, command: &GetCommand) -> Result<Option<String>, Error> {
        match self
            .storage
            .borrow_mut()
            .get(command.as_ref())
            .map(String::from)
        {
            Some(s) => Ok(Some(s)),
            None => Ok(None),
        }
    }
}
