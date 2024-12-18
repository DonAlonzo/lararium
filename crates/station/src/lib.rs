mod error;

pub use crate::error::Error;

use lararium_containers::ContainerRuntime;
use lararium_modules::{ModuleId, ModuleRuntime};

#[derive(Clone)]
pub struct Station {
    module_runtime: ModuleRuntime,
    container_runtime: ContainerRuntime,
}

impl Station {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            module_runtime: ModuleRuntime::new()?,
            container_runtime: ContainerRuntime::new()?,
        })
    }

    pub async fn add_module(
        &self,
        wasm: &[u8],
    ) -> Result<ModuleId, Error> {
        Ok(self.module_runtime.add_module(wasm).await?)
    }

    pub async fn remove_module(
        &self,
        id: ModuleId,
    ) -> Result<(), Error> {
        Ok(self.module_runtime.remove_module(id).await?)
    }
}
