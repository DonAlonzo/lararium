mod error;
pub use self::error::*;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::*;

use serde::{Deserialize, Serialize};

pub const JOIN_PATH: &str = "/join";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRequest {
    pub csr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResponse {
    pub certificate: String,
    pub ca: String,
}
