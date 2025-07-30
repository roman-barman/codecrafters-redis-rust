pub(crate) trait Command<T> {
    fn execute(&mut self) -> Result<T, String>;
}
