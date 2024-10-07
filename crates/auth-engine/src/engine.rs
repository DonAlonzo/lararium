use crate::{Error, Result};
use lararium::{Agent, LoginRequest, LoginResponse, Token};
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct Engine {
    pg_pool: PgPool,
}

#[derive(Clone, Copy)]
pub struct Transaction<'a, T> {
    pg_pool: &'a PgPool,
    context: T,
}

pub struct UnauthenticatedContext;

pub struct AuthenticatedContext;

impl Engine {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }

    pub fn unauthenticated(&self) -> Transaction<UnauthenticatedContext> {
        Transaction {
            pg_pool: &self.pg_pool,
            context: UnauthenticatedContext,
        }
    }

    pub fn authenticated(&self) -> Transaction<AuthenticatedContext> {
        Transaction {
            pg_pool: &self.pg_pool,
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
    pub async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<LoginResponse> {
        todo!();
    }
}

impl Transaction<'_, AuthenticatedContext> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_(pool: PgPool) -> sqlx::Result<()> {
        let engine = Engine::new(pool);
        Ok(())
    }
}
