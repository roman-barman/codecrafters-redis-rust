use crate::commands::Command;

pub trait CommandHandler<TCommand, TResult>: Send + Sync {
    fn handle(&self, command: TCommand) -> Result<TResult, anyhow::Error>
    where
        TCommand: Command<TResult>;
}
