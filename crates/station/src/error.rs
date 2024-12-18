use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    ModuleRuntime(lararium_modules::Error),
    #[from]
    ContainerRuntime(lararium_containers::Error),
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
