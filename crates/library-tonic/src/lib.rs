use lararium::*;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Library {}

impl Library {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl lararium::Library for Library {}
