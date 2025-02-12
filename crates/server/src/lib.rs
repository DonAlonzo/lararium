mod api;
mod dhcp;
mod dns;
mod error;
mod nfs;
mod ntp;
mod prelude;
mod stderr;
mod stdout;

use crypto::{Certificate, Identity};
use error::Error;
use stderr::StdErr;
use stdout::StdOut;

use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Result, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use std::future::Future;
use std::pin::Pin;

bindgen!({
    world: "application",
    async: true,
});

#[derive(Clone)]
pub struct Server {
    ca: Certificate,
    identity: Identity,
    engine: Engine,
    linker: Linker<State>,
}

impl Server {
    pub async fn new(
        ca: Certificate,
        identity: Identity,
    ) -> Result<Self, Error> {
        let engine = {
            let mut config = Config::new();
            config.async_support(true);
            Engine::new(&config)?
        };
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        Ok(Self {
            ca,
            identity,
            engine,
            linker,
        })
    }

    pub async fn run(
        &self,
        wasm: &[u8],
    ) -> Result<(), Error> {
        let component = Component::new(&self.engine, wasm)?;
        let ctx = WasiCtxBuilder::new()
            .stdout(StdOut::new())
            .stderr(StdErr::new())
            .env("USERNAME", "donalonzo")
            .allow_udp(true)
            .allow_tcp(true)
            .socket_addr_check(Box::new(|address, address_use| {
                Box::pin(async move {
                    tracing::info!("WASM connecting to {address}/{address_use:?}");
                    true
                }) as Pin<Box<dyn Future<Output = bool> + Send + Sync>>
            }))
            .build();
        let mut store = Store::new(
            &self.engine,
            State {
                ctx,
                table: ResourceTable::new(),
            },
        );
        let bindings = Application::instantiate_async(&mut store, &component, &self.linker).await?;
        bindings
            .call_run(&mut store)
            .await?
            .map_err(Error::Runtime)?;
        Ok(())
    }
}

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
