use crate::*;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use lararium_crypto::{Certificate, PrivateSignatureKey};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct Server {
    tcp_listener: TcpListener,
}

pub trait Handler {
    fn handle_join(
        &mut self,
        request: JoinRequest,
    ) -> impl std::future::Future<Output = Result<JoinResponse>> + Send;
}

async fn handle_join<T>(
    State(handler): State<Arc<Mutex<T>>>,
    Json(payload): Json<JoinRequest>,
) -> (StatusCode, Json<JoinResponse>)
where
    T: Handler + Clone + Send + Sync + 'static,
{
    let mut handler = handler.lock().await;
    let Ok(response) = handler.handle_join(payload).await else {
        todo!();
    };
    (StatusCode::OK, Json(response))
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
            .route(JOIN_PATH, post(handle_join::<T>))
            .with_state(shared_handler);
        axum::serve(self.tcp_listener, app).await.unwrap();
        Ok(())
    }
}
