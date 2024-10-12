use lararium::*;
use lararium_gateway_engine::{Engine, Error};
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Gateway {
    engine: Engine,
}

#[derive(Clone)]
pub struct Admittance {
    engine: Engine,
}

impl Gateway {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

impl Admittance {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

fn map_err(error: lararium_gateway_engine::Error) -> Status {
    match error {
        Error::InvalidCertificateSigningRequest => {
            Status::invalid_argument("invalid certificate signing request")
        }
    }
}

#[tonic::async_trait]
impl lararium::Admittance for Admittance {
    async fn join(
        &self,
        request: Request<JoinRequest>,
    ) -> Result<Response<JoinResponse>, Status> {
        self.engine
            .unauthenticated()
            .join(request.into_inner())
            .await
            .map(Response::new)
            .map_err(map_err)
    }
}

#[tonic::async_trait]
impl lararium::Gateway for Gateway {
    async fn check_in(
        &self,
        request: Request<CheckInRequest>,
    ) -> Result<Response<CheckInResponse>, Status> {
        self.engine
            .authenticated()
            .check_in(request.into_inner())
            .await
            .map(Response::new)
            .map_err(map_err)
    }

    async fn check_out(
        &self,
        request: Request<CheckOutRequest>,
    ) -> Result<Response<CheckOutResponse>, Status> {
        self.engine
            .authenticated()
            .check_out(request.into_inner())
            .await
            .map(Response::new)
            .map_err(map_err)
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        self.engine
            .authenticated()
            .heartbeat(request.into_inner())
            .await
            .map(Response::new)
            .map_err(map_err)
    }
}
