use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, From, PartialEq)]
pub enum Error {
    SubscriptionNotFound,
    EntryNotFound,
    Conflict,
    InvalidPayload,
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
