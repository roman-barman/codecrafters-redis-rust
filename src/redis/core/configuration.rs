use std::path::{Path, PathBuf};

pub struct Configuration {
    dir: Option<String>,
    db_file_name: Option<String>,
    port: u16,
}

impl Configuration {
    pub fn new(dir: Option<String>, db_file_name: Option<String>, port: u16) -> Self {
        Self {
            dir,
            db_file_name,
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn dir(&self) -> Option<&String> {
        self.dir.as_ref()
    }

    pub fn db_file_name(&self) -> Option<&String> {
        self.db_file_name.as_ref()
    }

    pub fn get_db_file_path(&self) -> Option<PathBuf> {
        if let Some(dir) = &self.dir {
            if let Some(db_file_name) = &self.db_file_name {
                return Some(Path::new(dir).join(db_file_name));
            }
        }
        None
    }
}
