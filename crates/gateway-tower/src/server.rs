use lararium::ClientInfo;
use lararium_crypto::Certificate;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tonic::body::BoxBody;
use tonic::transport::server::{TcpConnectInfo, TlsConnectInfo};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct ServerLayer {}

#[derive(Clone)]
pub struct ServerService<S> {
    inner: S,
}

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl ServerLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for ServerLayer {
    type Service = ServerService<S>;

    fn layer(
        &self,
        service: S,
    ) -> Self::Service {
        ServerService { inner: service }
    }
}

impl<S> Service<hyper::Request<BoxBody>> for ServerService<S>
where
    S: Service<hyper::Request<BoxBody>, Response = hyper::Response<BoxBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context)
    }

    fn call(
        &mut self,
        mut request: hyper::Request<BoxBody>,
    ) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let peer_cert = request
                .extensions()
                .get::<TlsConnectInfo<TcpConnectInfo>>()
                .and_then(|info| info.peer_certs())
                .and_then(|certs| certs.first().cloned());
            if let Some(peer_cert) = peer_cert {
                let Ok(peer_cert) = Certificate::from_der(&peer_cert) else {
                    todo!("handle invalid certificate");
                };
                if let Some(common_name) = peer_cert.common_name() {
                    request
                        .extensions_mut()
                        .insert(ClientInfo { name: common_name });
                };
            }
            let response = inner.call(request).await?;
            Ok(response)
        })
    }
}
