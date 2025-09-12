pub trait WriteResponse {
    type Error;
    fn write_simple_string(&mut self, message: impl AsRef<str>) -> Result<(), Self::Error>;
    fn write_error(&mut self, message: impl AsRef<str>) -> Result<(), Self::Error>;
    fn write_bulk_sting(&mut self, message: &Option<impl AsRef<str>>) -> Result<(), Self::Error>;
    fn write_array(&mut self, message: &Vec<Option<impl AsRef<str>>>) -> Result<(), Self::Error>;
}
