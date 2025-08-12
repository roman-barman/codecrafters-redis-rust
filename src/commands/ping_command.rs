use crate::commands::Command;

pub struct PingCommand;

impl PingCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command<String> for PingCommand {}
