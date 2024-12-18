mod error;

pub use error::Error;

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use wasmtime::*;

#[derive(Clone, Default)]
pub struct ModuleRuntime {
    engine: Engine,
    linker: Linker<CallState>,
    modules: Arc<DashMap<ModuleId, Module>>,
    next_module_id: Arc<AtomicU64>,
}

pub struct Module(wasmtime::Module);

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ModuleId(u64);

struct CallState {}

impl ModuleRuntime {
    pub fn new() -> Result<Self, Error> {
        let engine = {
            let mut config = Config::new();
            config.async_support(true);
            Engine::new(&config)?
        };
        let mut linker = Linker::new(&engine);
        linker.func_wrap_async("time", "sleep", {
            move |_caller: Caller<'_, CallState>, params: (u64,)| {
                Box::new(async move {
                    let (milliseconds,) = params;
                    tokio::time::sleep(tokio::time::Duration::from_millis(milliseconds)).await;
                })
            }
        })?;
        linker.func_wrap_async("tracing", "info", {
            move |mut caller: Caller<'_, CallState>, params: (u32, u32)| {
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
            move |mut caller: Caller<'_, CallState>, params: (u32, u32)| {
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
            move |mut caller: Caller<'_, CallState>, params: (u32, u32)| {
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
            move |mut caller: Caller<'_, CallState>, params: (u32, u32)| {
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
        let module = Module(wasmtime::Module::new(&self.engine, wasm)?);
        let module_id = ModuleId(self.next_module_id.fetch_add(1, Ordering::SeqCst));
        module.on_load(&self.engine, &self.linker).await?;
        self.modules.insert(module_id, module);
        Ok(module_id)
    }

    pub async fn remove_module(
        &self,
        id: ModuleId,
    ) -> Result<(), Error> {
        let Some((_, module)) = self.modules.remove(&id) else {
            return Err(Error::ModuleNotFound);
        };
        module.on_unload(&self.engine, &self.linker).await?;
        Ok(())
    }
}

impl Module {
    async fn on_load(
        &self,
        engine: &Engine,
        linker: &Linker<CallState>,
    ) -> Result<(), Error> {
        let mut store = Store::new(engine, CallState {});
        let instance = linker.instantiate_async(&mut store, &self.0).await?;
        let func = instance.get_typed_func::<_, ()>(&mut store, "on_load")?;
        func.call_async(&mut store, ()).await?;
        Ok(())
    }

    async fn on_unload(
        &self,
        engine: &Engine,
        linker: &Linker<CallState>,
    ) -> Result<(), Error> {
        let mut store = Store::new(engine, CallState {});
        let instance = linker.instantiate_async(&mut store, &self.0).await?;
        let func = instance.get_typed_func::<_, ()>(&mut store, "on_unload")?;
        func.call_async(&mut store, ()).await?;
        Ok(())
    }
}
