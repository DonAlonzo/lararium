use crate::{Error, Result};
use lararium::*;
use lararium_crypto::{CertificateSigningRequest, Identity};
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct Engine {
    pg_pool: PgPool,
    identity: Identity,
    ca: String,
}

#[derive(Clone, Copy)]
pub struct Transaction<'a, T> {
    pg_pool: &'a PgPool,
    identity: &'a Identity,
    ca: &'a str,
    context: T,
}

pub struct UnauthenticatedContext;

pub struct AuthenticatedContext;

impl Engine {
    pub fn new(
        pg_pool: PgPool,
        identity: Identity,
        ca: String,
    ) -> Self {
        Self {
            pg_pool,
            identity,
            ca,
        }
    }

    pub fn unauthenticated(&self) -> Transaction<UnauthenticatedContext> {
        Transaction {
            pg_pool: &self.pg_pool,
            identity: &self.identity,
            ca: &self.ca,
            context: UnauthenticatedContext,
        }
    }

    pub fn authenticated(&self) -> Transaction<AuthenticatedContext> {
        Transaction {
            pg_pool: &self.pg_pool,
            identity: &self.identity,
            ca: &self.ca,
            context: AuthenticatedContext,
        }
    }

    pub async fn authenticate(
        &self,
        token: Token,
    ) -> Result<Agent> {
        todo!();
    }
}

impl<T> Transaction<'_, T> {
    pub async fn join(
        &self,
        request: JoinRequest,
    ) -> Result<JoinResponse> {
        let Ok(csr) = CertificateSigningRequest::from_pem(request.csr.as_bytes()) else {
            return Err(Error::InvalidCertificateSigningRequest);
        };
        let Ok(certificate) = self.identity.sign_certificate_signing_request(&csr) else {
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
        println!("check_in");
        Ok(CheckInResponse {})
    }

    pub async fn check_out(
        &self,
        request: CheckOutRequest,
    ) -> Result<CheckOutResponse> {
        println!("check_out");
        Ok(CheckOutResponse {})
    }

    pub async fn heartbeat(
        &self,
        request: HeartbeatRequest,
    ) -> Result<HeartbeatResponse> {
        println!("heartbeat");
        Ok(HeartbeatResponse {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_(pool: PgPool) -> sqlx::Result<()> {
        let engine = Engine::new(pool);
        Ok(())
    }
}
