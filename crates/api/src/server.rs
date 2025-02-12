use crate::*;
use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Form, Json, Router,
};
use crypto::{Certificate, PrivateSignatureKey};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct Server {
    tcp_listener: TcpListener,
}

pub trait Handler {
    // fn handle_home(
    //     &self,
    // ) -> impl std::future::Future<Output = String> + Send;
}

impl Server {
    pub async fn bind(
        listen_address: SocketAddr,
        tls_private_key: PrivateSignatureKey,
        tls_certificate: Certificate,
    ) -> Result<Self> {
        let tcp_listener = tokio::net::TcpListener::bind(listen_address).await.unwrap();
        Ok(Self { tcp_listener })
    }

    pub async fn listen<T>(
        self,
        handler: T,
    ) -> Result<()>
    where
        T: Handler + Clone + Send + Sync + 'static,
    {
        let shared_handler = Arc::new(Mutex::new(handler));
        let app = Router::new()
            // .route("/", get(home::<T>))
            // .route("/login", post(login::<T>))
            .with_state(shared_handler);
        axum::serve(self.tcp_listener, app).await.unwrap();
        Ok(())
    }
}
