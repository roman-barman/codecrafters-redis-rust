pub struct Configuration {
    dir: Option<String>,
    db_file_name: Option<String>,
}

impl Configuration {
    pub fn new(dir: Option<String>, db_file_name: Option<String>) -> Self {
        Self { dir, db_file_name }
    }

    pub fn dir(&self) -> Option<&String> {
        self.dir.as_ref()
    }

    pub fn db_file_name(&self) -> Option<&String> {
        self.db_file_name.as_ref()
    }
}
