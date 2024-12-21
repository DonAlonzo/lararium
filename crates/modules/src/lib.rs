mod error;
mod stderr;
mod stdout;

pub use error::Error;

use crate::stderr::StdErr;
use crate::stdout::StdOut;
use dashmap::DashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use wasmtime::component::{Component, Linker, ResourceTable, TypedFunc};
use wasmtime::{Config, Engine, Result, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};

#[derive(Clone)]
pub struct ModuleRuntime {
    engine: Engine,
    linker: Linker<MyState>,
    modules: Arc<DashMap<ModuleId, Module>>,
    next_module_id: Arc<AtomicU64>,
}

pub struct Module {
    store: Store<MyState>,
    main: TypedFunc<(), ()>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ModuleId(u64);

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for MyState {
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
        Ok(Self {
            engine,
            linker,
            modules: Arc::new(DashMap::new()),
            next_module_id: Arc::new(AtomicU64::new(0)),
        })
    }

    pub async fn add_module(
        &self,
        wasm: &[u8],
    ) -> Result<ModuleId, Error> {
        let component = Component::new(&self.engine, wasm)?;
        let working_dir = std::path::Path::new("/tmp/floob");
        std::fs::create_dir_all(working_dir)?;
        let ctx = WasiCtxBuilder::new()
            .stdout(StdOut::new())
            .stderr(StdErr::new())
            .env("MQTT", "127.0.0.1:1883")
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
            MyState {
                ctx,
                table: ResourceTable::new(),
            },
        );
        let instance = self
            .linker
            .instantiate_async(&mut store, &component)
            .await?;
        let main = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .unwrap();
        let module_id = ModuleId(self.next_module_id.fetch_add(1, Ordering::SeqCst));
        let mut module = Module { store, main };
        module.main().await?;
        self.modules.insert(module_id, module);
        Ok(module_id)
    }

    pub async fn remove_module(
        &self,
        id: ModuleId,
    ) -> Result<(), Error> {
        let Some(_) = self.modules.remove(&id) else {
            return Err(Error::ModuleNotFound);
        };
        Ok(())
    }
}

impl Module {
    async fn main(&mut self) -> Result<(), Error> {
        self.main.call_async(&mut self.store, ()).await?;
        Ok(())
    }
}
