use crate::TOKEN;
use lararium::Token;
use lararium_controller_engine::Engine;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tonic::body::BoxBody;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct ServerLayer {
    controller_engine: Engine,
}

#[derive(Clone)]
pub struct ServerService<S> {
    inner: S,
    controller_engine: Engine,
}

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl ServerLayer {
    pub fn new(controller_engine: Engine) -> Self {
        Self { controller_engine }
    }
}

impl<S> Layer<S> for ServerLayer {
    type Service = ServerService<S>;

    fn layer(
        &self,
        service: S,
    ) -> Self::Service {
        ServerService {
            inner: service,
            controller_engine: self.controller_engine.clone(),
        }
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
        let controller_engine = self.controller_engine.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            let token = request.headers_mut().remove(TOKEN);
            request.headers_mut().clear();
            if let Some(token) = token {
                let Ok(token) = token.to_str().map(str::to_string) else {
                    todo!();
                };
                let token = Token::from(token);
                let agent = match controller_engine.authenticate(token).await {
                    Ok(agent) => agent,
                    Err(_) => todo!(),
                };
                request.extensions_mut().insert(agent);
            }
            let response = inner.call(request).await?;
            Ok(response)
        })
    }
}
