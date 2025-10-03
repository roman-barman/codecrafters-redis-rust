pub trait ReadResp {
    type Error;
    fn read_resp(&self) -> Result<Vec<String>, Self::Error>;
}
