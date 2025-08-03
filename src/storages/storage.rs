pub trait Storage: Send + Sync {
    fn get(&self, key: &str) -> Option<&str>;
    fn set(&mut self, key: &str, value: &str);
}