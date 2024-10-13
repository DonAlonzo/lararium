use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tonic::body::BoxBody;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct ClientLayer {}

#[derive(Clone)]
pub struct ClientService<S> {
    inner: S,
}

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl ClientLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for ClientLayer {
    type Service = ClientService<S>;

    fn layer(
        &self,
        service: S,
    ) -> Self::Service {
        ClientService { inner: service }
    }
}

impl<S> Service<hyper::Request<BoxBody>> for ClientService<S>
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
        request: hyper::Request<BoxBody>,
    ) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let response = inner.call(request).await?;
            Ok(response)
        })
    }
}
