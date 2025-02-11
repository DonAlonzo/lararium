mod api;
mod dhcp;
mod dns;
mod nfs;
mod ntp;
mod prelude;

use lararium_crypto::{Certificate, Identity};

#[derive(Clone)]
pub struct Server {
    ca: Certificate,
    identity: Identity,
}

impl Server {
    pub async fn new(
        ca: Certificate,
        identity: Identity,
    ) -> Self {
        Self { ca, identity }
    }
}
