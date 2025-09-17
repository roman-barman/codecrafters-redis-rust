use std::path::Path;

pub trait Storage {
    fn get(&mut self, key: &str) -> Option<&str>;
    fn set(&mut self, key: String, value: String, px: Option<u64>);
    fn get_keys(&mut self) -> Vec<&str>;
    fn save(&self, path: &Path) -> Result<(), std::io::Error>;
}
