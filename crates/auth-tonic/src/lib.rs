use lararium::*;
use lararium_auth_engine::Engine;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Server {
    engine: Engine,
}

impl Server {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

fn map_err(error: lararium_auth_engine::Error) -> Status {
    match error {
        //_ => Status::internal("engine error"),
    }
}

#[tonic::async_trait]
impl Auth for Server {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        self.engine
            .unauthenticated()
            .login(request.into_inner())
            .await
            .map(Response::new)
            .map_err(map_err)
    }
}
