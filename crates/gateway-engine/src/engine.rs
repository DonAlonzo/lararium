use crate::{Error, Result};
use lararium::*;
use lararium_crypto::{CertificateSigningRequest, Identity};

#[derive(Clone)]
pub struct Engine {
    identity: Identity,
    ca: String,
}

#[derive(Clone, Copy)]
pub struct Transaction<'a, T> {
    identity: &'a Identity,
    ca: &'a str,
    context: T,
}

pub struct UnauthenticatedContext;

pub struct AuthenticatedContext {
    client_info: ClientInfo,
}

impl Engine {
    pub fn new(
        identity: Identity,
        ca: String,
    ) -> Self {
        Self { identity, ca }
    }

    pub fn unauthenticated(&self) -> Transaction<UnauthenticatedContext> {
        Transaction {
            identity: &self.identity,
            ca: &self.ca,
            context: UnauthenticatedContext,
        }
    }

    pub fn authenticated(
        &self,
        client_info: ClientInfo,
    ) -> Transaction<AuthenticatedContext> {
        Transaction {
            identity: &self.identity,
            ca: &self.ca,
            context: AuthenticatedContext { client_info },
        }
    }
}

impl Transaction<'_, UnauthenticatedContext> {
    pub async fn join(
        &self,
        request: JoinRequest,
    ) -> Result<JoinResponse> {
        let Ok(csr) = CertificateSigningRequest::from_pem(request.csr.as_bytes()) else {
            return Err(Error::InvalidCertificateSigningRequest);
        };
        let Ok(certificate) = self.identity.sign_csr(&csr, "random-name") else {
            return Err(Error::InvalidCertificateSigningRequest);
        };
        let Ok(certificate) = certificate.to_pem() else {
            return Err(Error::InvalidCertificateSigningRequest);
        };
        let Ok(certificate) = String::from_utf8(certificate) else {
            return Err(Error::InvalidCertificateSigningRequest);
        };
        Ok(JoinResponse {
            certificate,
            ca: self.ca.into(),
        })
    }
}

impl Transaction<'_, AuthenticatedContext> {
    pub async fn check_in(
        &self,
        request: CheckInRequest,
    ) -> Result<CheckInResponse> {
        tracing::info!("[{}] checked in", self.context.client_info.name);
        Ok(CheckInResponse {})
    }

    pub async fn check_out(
        &self,
        request: CheckOutRequest,
    ) -> Result<CheckOutResponse> {
        tracing::info!("[{}] checked out", self.context.client_info.name);
        Ok(CheckOutResponse {})
    }

    pub async fn heartbeat(
        &self,
        request: HeartbeatRequest,
    ) -> Result<HeartbeatResponse> {
        tracing::info!("[{}] sent heartbeat", self.context.client_info.name);
        Ok(HeartbeatResponse {})
    }
}

#[cfg(test)]
mod tests {}
