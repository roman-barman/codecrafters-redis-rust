use crate::commands::Command;
use crate::handlers::CommandHandler;
use anyhow::{anyhow, Error};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct Mediator {
    handlers: HashMap<TypeId, Box<dyn ErasedHandler>>,
}

impl Mediator {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<TCommand, TResult, THandler>(&mut self, handler: THandler)
    where
        TCommand: Command<TResult> + 'static + Send + Sync,
        TResult: 'static + Send + Sync,
        THandler: CommandHandler<TCommand, TResult> + 'static + Send + Sync,
    {
        let handler = HandlerAdapter::new(handler);
        self.handlers.insert(TypeId::of::<TCommand>(), Box::new(handler));
    }

    pub fn send<TCommand, TResult>(&self, command: TCommand) -> Result<TResult, Error>
    where
        TCommand: Command<TResult> + 'static + Send + Sync,
        TResult: 'static + Send + Sync,
    {
        let handler = self.handlers.get(&TypeId::of::<TCommand>())
            .ok_or_else(|| anyhow!("No handler registered for this command type"))?;
        let result = handler.handle(Box::new(command))?;
        result.downcast::<TResult>()
            .map(|boxed| *boxed)
            .map_err(|_| anyhow!("Handler returned unexpected result type"))
    }
}

trait ErasedHandler: Send + Sync {
    fn handle(&self, command: Box<dyn Any>) -> Result<Box<dyn Any>, Error>;
}

struct HandlerAdapter<TCommand, TResult, THandler>
where
    TCommand: Command<TResult> + 'static + Send + Sync,
    TResult: 'static + Send + Sync,
    THandler: CommandHandler<TCommand, TResult> + 'static + Send + Sync,
{
    handler: THandler,
    _pd: PhantomData<(TCommand, TResult)>,
}

impl<TCommand, TResult, THandler> HandlerAdapter<TCommand, TResult, THandler>
where
    TCommand: Command<TResult> + 'static + Send + Sync,
    TResult: 'static + Send + Sync,
    THandler: CommandHandler<TCommand, TResult> + 'static + Send + Sync,
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
    TCommand: Command<TResult> + 'static + Send + Sync,
    TResult: 'static + Send + Sync,
    THandler: CommandHandler<TCommand, TResult> + 'static + Send + Sync,
{
    fn handle(&self, command: Box<dyn Any>) -> Result<Box<dyn Any>, Error> {
        let command = command.downcast::<TCommand>()
            .map_err(|_| anyhow!("Invalid command type for this handler"))?;
        let result = self.handler.handle(*command)?;
        Ok(Box::new(result))
    }
}
