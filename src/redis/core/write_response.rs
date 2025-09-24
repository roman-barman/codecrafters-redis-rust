pub trait WriteResponse {
    fn write_simple_string(&mut self, message: impl AsRef<str>) -> std::io::Result<()>;
    fn write_error(&mut self, message: impl AsRef<str>) -> std::io::Result<()>;
    fn write_bulk_sting(&mut self, message: &Option<impl AsRef<str>>) -> std::io::Result<()>;
    fn write_array(&mut self, message: &[Option<impl AsRef<str>>]) -> std::io::Result<()>;
}
