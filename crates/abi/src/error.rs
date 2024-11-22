pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidData,
}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
