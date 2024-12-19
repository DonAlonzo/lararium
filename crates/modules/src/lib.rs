mod error;
mod stderr;
mod stdout;

pub use error::Error;

use crate::stderr::StdErr;
use crate::stdout::StdOut;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use wasmtime::*;
use wasmtime_wasi::{preview1::WasiP1Ctx, DirPerms, FilePerms, WasiCtxBuilder};

#[derive(Clone, Default)]
pub struct ModuleRuntime {
    engine: Engine,
    linker: Linker<WasiP1Ctx>,
    modules: Arc<DashMap<ModuleId, Module>>,
    next_module_id: Arc<AtomicU64>,
}

pub struct Module {
    store: wasmtime::Store<WasiP1Ctx>,
    on_load: wasmtime::TypedFunc<(), ()>,
    on_publish: wasmtime::TypedFunc<(), ()>,
    on_unload: wasmtime::TypedFunc<(), ()>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ModuleId(u64);

impl ModuleRuntime {
    pub fn new() -> Result<Self, Error> {
        let engine = {
            let mut config = Config::new();
            config.async_support(true);
            Engine::new(&config)?
        };
        let mut linker = Linker::<WasiP1Ctx>::new(&engine);
        wasmtime_wasi::preview1::add_to_linker_async(&mut linker, |t| t)?;

        linker.func_wrap_async("time", "sleep", {
            move |_caller: Caller<'_, _>, params: (u64,)| {
                Box::new(async move {
                    let (milliseconds,) = params;
                    tokio::time::sleep(tokio::time::Duration::from_millis(milliseconds)).await;
                })
            }
        })?;
        linker.func_wrap_async("tracing", "info", {
            move |mut caller: Caller<'_, _>, params: (u32, u32)| {
                Box::new(async move {
                    let (message, message_len) = params;
                    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                        return;
                    };
                    let memory_data = memory.data(&caller);
                    let Some(message) = memory_data
                        .get(message as usize..(message + message_len) as usize)
                        .and_then(|s| std::str::from_utf8(s).ok())
                    else {
                        return;
                    };
                    tracing::info!("{message}");
                })
            }
        })?;
        linker.func_wrap_async("tracing", "debug", {
            move |mut caller: Caller<'_, _>, params: (u32, u32)| {
                Box::new(async move {
                    let (message, message_len) = params;
                    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                        return;
                    };
                    let memory_data = memory.data(&caller);
                    let Some(message) = memory_data
                        .get(message as usize..(message + message_len) as usize)
                        .and_then(|s| std::str::from_utf8(s).ok())
                    else {
                        return;
                    };
                    tracing::debug!("{message}");
                })
            }
        })?;
        linker.func_wrap_async("tracing", "warn", {
            move |mut caller: Caller<'_, _>, params: (u32, u32)| {
                Box::new(async move {
                    let (message, message_len) = params;
                    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                        return;
                    };
                    let memory_data = memory.data(&caller);
                    let Some(message) = memory_data
                        .get(message as usize..(message + message_len) as usize)
                        .and_then(|s| std::str::from_utf8(s).ok())
                    else {
                        return;
                    };
                    tracing::warn!("{message}");
                })
            }
        })?;
        linker.func_wrap_async("tracing", "error", {
            move |mut caller: Caller<'_, _>, params: (u32, u32)| {
                Box::new(async move {
                    let (message, message_len) = params;
                    let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                        return;
                    };
                    let memory_data = memory.data(&caller);
                    let Some(message) = memory_data
                        .get(message as usize..(message + message_len) as usize)
                        .and_then(|s| std::str::from_utf8(s).ok())
                    else {
                        return;
                    };
                    tracing::error!("{message}");
                })
            }
        })?;
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
        let module = wasmtime::Module::new(&self.engine, wasm)?;
        let working_dir = std::path::Path::new("/tmp/floob");
        std::fs::create_dir_all(working_dir)?;
        let wasi = WasiCtxBuilder::new()
            .stdout(StdOut::new())
            .stderr(StdErr::new())
            .env("FOO", "bar")
            .preopened_dir(working_dir, "/", DirPerms::all(), FilePerms::all())?
            .build_p1();
        let mut store = Store::new(&self.engine, wasi);
        let instance = self.linker.instantiate_async(&mut store, &module).await?;
        let on_load = instance.get_typed_func::<(), ()>(&mut store, "on_load")?;
        let on_publish = instance.get_typed_func::<(), ()>(&mut store, "on_publish")?;
        let on_unload = instance.get_typed_func::<(), ()>(&mut store, "on_unload")?;
        let module_id = ModuleId(self.next_module_id.fetch_add(1, Ordering::SeqCst));
        let mut module = Module {
            store,
            on_load,
            on_publish,
            on_unload,
        };
        module.on_load().await?;
        self.modules.insert(module_id, module);
        Ok(module_id)
    }

    pub async fn remove_module(
        &self,
        id: ModuleId,
    ) -> Result<(), Error> {
        let Some((_, mut module)) = self.modules.remove(&id) else {
            return Err(Error::ModuleNotFound);
        };
        module.on_unload().await?;
        Ok(())
    }
}

impl Module {
    async fn on_load(&mut self) -> Result<(), Error> {
        self.on_load.call_async(&mut self.store, ()).await?;
        Ok(())
    }

    async fn on_publish(&mut self) -> Result<(), Error> {
        self.on_publish.call_async(&mut self.store, ()).await?;
        Ok(())
    }

    async fn on_unload(&mut self) -> Result<(), Error> {
        self.on_unload.call_async(&mut self.store, ()).await?;
        Ok(())
    }
}
