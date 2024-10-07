use crate::TOKEN;
use lararium::Token;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tonic::body::BoxBody;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct ClientLayer {
    token: Option<Token>,
}

#[derive(Clone)]
pub struct ClientService<S> {
    inner: S,
    token: Option<Token>,
}

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl ClientLayer {
    pub fn new(token: Option<Token>) -> Self {
        Self { token }
    }
}

impl<S> Layer<S> for ClientLayer {
    type Service = ClientService<S>;

    fn layer(
        &self,
        service: S,
    ) -> Self::Service {
        ClientService {
            inner: service,
            token: self.token.clone(),
        }
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
        mut request: hyper::Request<BoxBody>,
    ) -> Self::Future {
        let clone = self.inner.clone();
        let token = self.token.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            if let Some(token) = token {
                let Ok(token) = token.to_string().parse() else {
                    todo!();
                };
                request.headers_mut().insert(TOKEN, token);
            }
            let response = inner.call(request).await?;
            Ok(response)
        })
    }
}
