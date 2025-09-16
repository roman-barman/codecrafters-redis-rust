use std::path::PathBuf;

pub trait Storage {
    fn get(&mut self, key: &str) -> Option<&str>;
    fn set(&mut self, key: String, value: String, px: Option<u64>);
    fn get_keys(&mut self) -> Vec<&str>;
    fn save(&self, path: &PathBuf) -> Result<(), std::io::Error>;
}
