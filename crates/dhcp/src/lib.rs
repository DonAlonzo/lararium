#[cfg(feature = "client")]
mod client;
mod error;
#[cfg(feature = "server")]
mod server;

pub use self::error::{Error, Result};
#[cfg(feature = "client")]
pub use client::Client;
#[cfg(feature = "server")]
pub use server::Server;
