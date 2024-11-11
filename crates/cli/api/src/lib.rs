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

use lararium_crypto::{Certificate, CertificateSigningRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequest {
    pub csr: CertificateSigningRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinResponse {
    pub certificate: Certificate,
    pub ca: Certificate,
}
