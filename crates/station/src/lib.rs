mod error;

pub use crate::error::Error;

use lararium_containers::ContainerRuntime;
use lararium_modules::ModuleRuntime;

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

    pub async fn run(
        &self,
        wasm: &[u8],
    ) -> Result<(), Error> {
        Ok(self.module_runtime.run(wasm).await?)
    }
}
