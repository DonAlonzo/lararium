mod error;
mod stderr;
mod stdout;

pub use error::Error;

use crate::modules::extension::*;
use crate::stderr::StdErr;
use crate::stdout::StdOut;
use std::future::Future;
use std::pin::Pin;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Result, Store};
use wasmtime_wasi::{async_trait, DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};

bindgen!({
    world: "extension",
    async: true,
});

#[derive(Clone)]
pub struct ModuleRuntime {
    engine: Engine,
    linker: Linker<State>,
}

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

#[async_trait]
impl oci::Host for State {
    async fn download_image(
        &mut self,
        reference: String,
    ) {
        tracing::debug!("WASM called download_image");
    }

    async fn run_container(&mut self) {
        tracing::debug!("WASM called run_container");
    }
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl ModuleRuntime {
    pub fn new() -> Result<Self, Error> {
        let engine = {
            let mut config = Config::new();
            config.async_support(true);
            Engine::new(&config)?
        };
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        Extension::add_to_linker(&mut linker, |s| s)?;
        Ok(Self { engine, linker })
    }

    pub async fn run(
        &self,
        wasm: &[u8],
    ) -> Result<(), Error> {
        let component = Component::new(&self.engine, wasm)?;
        let working_dir = std::path::Path::new("/tmp/floob");
        std::fs::create_dir_all(working_dir)?;
        let ctx = WasiCtxBuilder::new()
            .stdout(StdOut::new())
            .stderr(StdErr::new())
            .env("GATEWAY", "127.0.0.1")
            .env("MQTT_PORT", "1883")
            .allow_udp(true)
            .allow_tcp(true)
            .socket_addr_check(Box::new(|address, address_use| {
                Box::pin(async move {
                    tracing::info!("WASM connecting to {address}/{address_use:?}");
                    true
                }) as Pin<Box<dyn Future<Output = bool> + Send + Sync>>
            }))
            .preopened_dir(working_dir, "/", DirPerms::all(), FilePerms::all())?
            .build();
        let mut store = Store::new(
            &self.engine,
            State {
                ctx,
                table: ResourceTable::new(),
            },
        );
        let bindings = Extension::instantiate_async(&mut store, &component, &self.linker).await?;
        bindings
            .call_run(&mut store)
            .await?
            .map_err(Error::Runtime)?;
        Ok(())
    }
}
