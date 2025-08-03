pub(crate) struct PingCommand;

impl PingCommand {
    pub(crate) fn execute(&mut self) -> String {
        "PONG".to_string()
    }
}
