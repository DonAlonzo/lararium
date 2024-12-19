use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    ModuleNotFound,
    #[from]
    Wasm(wasmtime::Error),
    #[from]
    Io(std::io::Error),
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
