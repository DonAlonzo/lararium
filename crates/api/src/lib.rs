mod error;
pub use self::error::*;

use lararium::prelude::*;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::*;

#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::*;

use lararium_crypto::{Certificate, CertificateSigningRequest};
use serde::{Deserialize, Serialize};

pub const CONTENT_TYPE_SIGNAL: &str = "application/vnd.lararium.signal";
pub const CONTENT_TYPE_BOOLEAN: &str = "application/vnd.lararium.boolean";

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequest {
    pub csr: CertificateSigningRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinResponse {
    pub certificate: Certificate,
    pub ca: Certificate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRequest {
    pub topic: Topic,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    pub entry: Entry,
}
