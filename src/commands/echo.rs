pub(crate) struct EchoCommand;

impl EchoCommand {
    pub(crate) fn execute(&mut self, arg: &str) -> String {
        arg.to_string()
    }
}
