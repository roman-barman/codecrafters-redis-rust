use crate::commands::Command;

pub trait CommandHandler<TCommand, TResult> {
    fn handle(&self, command: TCommand) -> Result<TResult, anyhow::Error>
    where
        TCommand: Command<TResult>;
}
