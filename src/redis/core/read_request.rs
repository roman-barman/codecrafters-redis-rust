pub trait ReadRequest {
    type Error;
    fn read_request(&self) -> Result<Vec<String>, Self::Error>;
}
