use crate::commands::Command;
use crate::handlers::CommandHandler;
use crate::mediators::mediator::Mediator;
use anyhow::{anyhow, Error};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct MediatorImpl {
    handlers: HashMap<TypeId, Box<dyn ErasedHandler>>,
}

impl MediatorImpl {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<TCommand, TResult, THandler>(&mut self, handler: THandler)
    where
        TCommand: Command<TResult> + 'static,
        TResult: 'static,
        THandler: CommandHandler<TCommand, TResult> + 'static,
    {
        let handler = HandlerAdapter::new(handler);
        self.handlers.insert(TypeId::of::<TCommand>(), Box::new(handler));
    }
}

impl Mediator for MediatorImpl {
    fn send<TCommand, TResult>(&self, command: TCommand) -> Result<TResult, Error>
    where
        TCommand: Command<TResult> + 'static,
        TResult: 'static,
    {
        let handler = self.handlers.get(&TypeId::of::<TCommand>())
            .ok_or_else(|| anyhow!("No handler registered for this command type"))?;
        let result = handler.handle(Box::new(command))?;
        result.downcast::<TResult>()
            .map(|boxed| *boxed)
            .map_err(|_| anyhow!("Handler returned unexpected result type"))
    }
}

trait ErasedHandler {
    fn handle(&self, command: Box<dyn Any>) -> Result<Box<dyn Any>, Error>;
}

struct HandlerAdapter<TCommand, TResult, THandler>
where
    TCommand: Command<TResult> + 'static,
    TResult: 'static,
    THandler: CommandHandler<TCommand, TResult> + 'static,
{
    handler: THandler,
    _pd: PhantomData<(TCommand, TResult)>,
}

impl<TCommand, TResult, THandler> HandlerAdapter<TCommand, TResult, THandler>
where
    TCommand: Command<TResult> + 'static,
    TResult: 'static,
    THandler: CommandHandler<TCommand, TResult> + 'static,
{
    fn new(handler: THandler) -> Self {
        Self {
            handler,
            _pd: PhantomData,
        }
    }
}

impl<TCommand, TResult, THandler> ErasedHandler for HandlerAdapter<TCommand, TResult, THandler>
where
    TCommand: Command<TResult> + 'static,
    TResult: 'static,
    THandler: CommandHandler<TCommand, TResult> + 'static,
{
    fn handle(&self, command: Box<dyn Any>) -> Result<Box<dyn Any>, Error> {
        let command = command.downcast::<TCommand>()
            .map_err(|_| anyhow!("Invalid command type for this handler"))?;
        let result = self.handler.handle(*command)?;
        Ok(Box::new(result))
    }
}
