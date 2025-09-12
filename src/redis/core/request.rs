#[derive(Debug)]
pub struct Request {
    value: Vec<String>,
}

impl Request {
    pub fn new(value: Vec<String>) -> Self {
        Self { value }
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.value.get(index)
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }
}
