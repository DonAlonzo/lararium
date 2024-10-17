#[cfg(feature = "client")]
mod client;
mod error;
mod protocol;
#[cfg(feature = "server")]
mod server;

pub use self::error::{Error, Result};
#[cfg(feature = "client")]
pub use client::Client;
pub use protocol::{Answer, Class, OperationCode, Query, RecordType, Response, ResponseCode};
#[cfg(feature = "server")]
pub use server::{Handler, Server};
