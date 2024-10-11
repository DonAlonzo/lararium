mod error;

pub use self::error::{Error, Result};

use lararium::*;
use lararium_crypto::Certificate;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Server {
    tx: mpsc::Sender<Certificate>,
    rx: Arc<Mutex<mpsc::Receiver<Certificate>>>,
}

impl Server {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        Self {
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub async fn wait_for_adoption(&self) -> Result<Certificate> {
        Ok(self.rx.lock().await.recv().await.unwrap())
    }
}

#[tonic::async_trait]
impl Adoption for Server {
    async fn propose(
        &self,
        request: Request<ProposeRequest>,
    ) -> std::result::Result<Response<ProposeResponse>, Status> {
        todo!();
        Ok(Response::new(ProposeResponse { csr: "csr".into() }))
    }

    async fn accept(
        &self,
        request: Request<AcceptRequest>,
    ) -> std::result::Result<Response<AcceptResponse>, Status> {
        match self.tx.send(todo!()).await {
            Ok(_) => Ok(Response::new(AcceptResponse {})),
            Err(_) => Err(Status::internal("channel closed")),
        }
    }
}
