#[cfg(feature = "client")]
mod client;
mod error;

pub use self::error::{Error, Result};
#[cfg(feature = "client")]
pub use client::Client;
