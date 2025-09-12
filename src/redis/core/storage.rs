pub trait Storage {
    fn get(&mut self, key: &str) -> Option<&str>;
    fn set(&mut self, key: String, value: String, px: Option<i64>);
}
