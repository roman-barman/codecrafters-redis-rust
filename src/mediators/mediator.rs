use crate::commands::Command;

pub trait Mediator {
    fn send<TCommand, TResult>(&self, command: TCommand) -> Result<TResult, anyhow::Error>
    where
        TCommand: Command<TResult> + 'static,
        TResult: 'static,;
}
