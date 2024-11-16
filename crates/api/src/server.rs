use crate::*;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
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
        &self,
        request: JoinRequest,
    ) -> impl std::future::Future<Output = Result<JoinResponse>> + Send;

    fn handle_registry_read(
        &self,
        request: GetRequest,
    ) -> impl std::future::Future<Output = Result<GetResponse>> + Send;
}

async fn join<T>(
    State(handler): State<Arc<Mutex<T>>>,
    Json(payload): Json<JoinRequest>,
) -> (StatusCode, Json<JoinResponse>)
where
    T: Handler + Clone + Send + Sync + 'static,
{
    let handler = handler.lock().await;
    let Ok(response) = handler.handle_join(payload).await else {
        todo!();
    };
    (StatusCode::OK, Json(response))
}

async fn registry_read<T>(
    State(handler): State<Arc<Mutex<T>>>,
    Path(suffix): Path<String>,
) -> impl IntoResponse
where
    T: Handler + Clone + Send + Sync + 'static,
{
    let handler = handler.lock().await;
    let request = GetRequest { key: suffix };
    let Ok(response) = handler.handle_registry_read(request).await else {
        todo!();
    };
    match response.entry {
        Entry::Signal => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, CONTENT_TYPE_SIGNAL.parse().unwrap());
            (StatusCode::OK, headers, vec![])
        }
        Entry::Boolean(value) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, CONTENT_TYPE_BOOLEAN.parse().unwrap());
            (StatusCode::OK, headers, vec![value as u8])
        }
    }
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
            .route("/join", post(join::<T>))
            .route("/~/*key", get(registry_read::<T>))
            .with_state(shared_handler);
        axum::serve(self.tcp_listener, app).await.unwrap();
        Ok(())
    }
}
