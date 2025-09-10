use std::collections::HashMap;

pub struct RedisDatabase {
    version: String,
    metadata: HashMap<String, String>,
    databases: Vec<Database>,
    checksum: u64,
}

struct Database {
    number: u32,
    data: HashMap<String, (String, Option<u64>)>,
}
