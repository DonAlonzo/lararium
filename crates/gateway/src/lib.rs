mod api;
mod dhcp;
mod dns;
mod mqtt;
mod prelude;

use dashmap::DashMap;
use lararium::prelude::*;
use lararium_crypto::{Certificate, Identity};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::*;

#[derive(Clone)]
pub struct Gateway {
    core: Arc<RwLock<Core>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Subscriber {
    Client(u64),
    Module(u64),
}

#[derive(Clone)]
struct Core {
    ca: Certificate,
    identity: Identity,
    engine: Engine,
    linker: Linker<CallState>,
    modules: Arc<DashMap<u64, Module>>,
    next_module_id: Arc<AtomicU64>,
    registry: Arc<lararium_registry::Registry<Subscriber>>,
    mqtt: lararium_mqtt::Server<Gateway>,
    dns: lararium_dns::Server,
    dhcp: lararium_dhcp::Server,
}

struct CallState {}

impl Gateway {
    pub async fn new(
        ca: Certificate,
        identity: Identity,
        mqtt: lararium_mqtt::Server<Gateway>,
        dns: lararium_dns::Server,
        dhcp: lararium_dhcp::Server,
    ) -> Self {
        let core = Arc::new(RwLock::new(Core::new(ca, identity, mqtt, dns, dhcp)));
        core.write().await.link(core.clone());
        Self { core }
    }
}

trait Linkage {
    fn registry_read(
        &self,
        topic: Topic,
    ) -> impl std::future::Future<Output = Result<Entry>> + Send;

    fn registry_write(
        &self,
        topic: Topic,
        value: Value,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn registry_delete(
        &self,
        topic: Topic,
    ) -> impl std::future::Future<Output = Result<Entry>> + Send;
}

impl Core {
    pub fn new(
        ca: Certificate,
        identity: Identity,
        mqtt: lararium_mqtt::Server<Gateway>,
        dns: lararium_dns::Server,
        dhcp: lararium_dhcp::Server,
    ) -> Self {
        let engine = {
            let mut config = Config::new();
            config.async_support(true);
            Engine::new(&config).unwrap()
        };
        let linker = Linker::new(&engine);
        let registry = Arc::new(lararium_registry::Registry::new());
        Self {
            ca,
            identity,
            engine,
            linker,
            modules: Arc::new(DashMap::new()),
            next_module_id: Arc::new(AtomicU64::new(0)),
            registry,
            mqtt,
            dns,
            dhcp,
        }
    }

    pub fn add_module(
        &mut self,
        wasm: &[u8],
        subscriptions: &[Filter],
    ) {
        let module = Module::new(&self.engine, wasm).unwrap();
        let module_id = self.next_module_id.fetch_add(1, Ordering::SeqCst);
        self.modules.insert(module_id, module);
        for subscription in subscriptions {
            self.registry
                .subscribe(Subscriber::Module(module_id), subscription)
                .unwrap();
        }
    }

    pub async fn registry_read(
        &self,
        topic: Topic,
    ) -> Result<Entry> {
        Ok(self.registry.read(&topic)?)
    }

    pub async fn registry_write(
        &self,
        topic: Topic,
        value: Value,
    ) -> Result<()> {
        let (subscribers, _) = self.registry.update(&topic, value.clone())?;
        let mut client_ids = vec![];
        let mut module_ids = vec![];
        for subscriber in subscribers.into_iter() {
            match subscriber {
                Subscriber::Client(client_id) => client_ids.push(client_id),
                Subscriber::Module(module_id) => module_ids.push(module_id),
            }
        }
        self.mqtt
            .publish(&client_ids, &topic, Some(value.clone()))
            .await
            .unwrap();
        self.on_registry_write(&topic, value, &module_ids).await;
        Ok(())
    }

    async fn on_registry_write(
        &self,
        topic: &Topic,
        value: Value,
        module_ids: &[u64],
    ) {
        for module_id in module_ids {
            let module = self.modules.get(module_id).unwrap();
            let mut store = Store::new(&self.engine, CallState {});
            let instance = self
                .linker
                .instantiate_async(&mut store, &module)
                .await
                .unwrap();
            let Ok(run) = instance.get_typed_func::<_, ()>(&mut store, "on_registry_write") else {
                continue;
            };
            let memory = instance.get_memory(&mut store, "memory").unwrap();
            let write_to_memory = |data: &[u8], memory: &Memory, store: &mut Store<CallState>| {
                let ptr = memory.size(&mut *store) as u32 * 65536;
                memory
                    .grow(&mut *store, ((data.len() + 0xffff) / 65536) as u64)
                    .unwrap();
                let memory = memory.data_mut(store);
                memory[ptr as usize..ptr as usize + data.len()].copy_from_slice(data);
                ptr
            };
            let topic = topic.to_string();
            let mut payload = Vec::new();
            ciborium::ser::into_writer(&value, &mut payload).unwrap();
            let topic_ptr = write_to_memory(topic.as_bytes(), &memory, &mut store);
            let payload_ptr = write_to_memory(&payload, &memory, &mut store);
            tokio::task::spawn(async move {
                run.call_async(
                    &mut store,
                    (
                        topic_ptr,
                        topic.len() as u32,
                        payload_ptr,
                        payload.len() as u32,
                    ),
                )
                .await
                .unwrap();
            });
        }
    }

    pub async fn registry_delete(
        &self,
        topic: Topic,
    ) -> Result<Entry> {
        let (subscribers, old_entry) = self.registry.delete(&topic)?;
        let mut client_ids = vec![];
        let mut module_ids = vec![];
        for subscriber in subscribers.into_iter() {
            match subscriber {
                Subscriber::Client(client_id) => client_ids.push(client_id),
                Subscriber::Module(module_id) => module_ids.push(module_id),
            }
        }
        self.mqtt.publish(&client_ids, &topic, None).await.unwrap();
        Ok(old_entry)
    }

    pub fn link<T>(
        &mut self,
        link: T,
    ) where
        T: Linkage + Clone + Send + Sync + 'static,
    {
        self.linker
            .func_wrap_async("time", "sleep", {
                move |_caller: Caller<'_, CallState>, params: (u64,)| {
                    Box::new(async move {
                        let (milliseconds,) = params;
                        tokio::time::sleep(tokio::time::Duration::from_millis(milliseconds)).await;
                    })
                }
            })
            .unwrap();
        self.linker
            .func_wrap_async("tracing", "info", {
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
            })
            .unwrap();
        self.linker
            .func_wrap_async("tracing", "debug", {
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
            })
            .unwrap();
        self.linker
            .func_wrap_async("tracing", "warn", {
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
            })
            .unwrap();
        self.linker
            .func_wrap_async("tracing", "error", {
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
            })
            .unwrap();
        self.linker
            .func_wrap_async("registry", "read", {
                let link = link.clone();
                move |mut caller: Caller<'_, CallState>, params: (u32, u32, u32, u32)| {
                    let link = link.clone();
                    Box::new(async move {
                        let (topic, topic_len, buffer, buffer_len) = params;
                        let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                            return u32::MAX;
                        };
                        let memory_data = memory.data(&caller);
                        let Some(topic) = memory_data
                            .get(topic as usize..(topic + topic_len) as usize)
                            .and_then(|s| std::str::from_utf8(s).ok())
                        else {
                            return u32::MAX;
                        };
                        let topic = Topic::from_str(topic);
                        let Ok(Entry::Record { value, .. }) =
                            link.registry_read(topic.clone()).await
                        else {
                            return u32::MAX;
                        };
                        let mut cbor = Vec::new();
                        ciborium::ser::into_writer(&value, &mut cbor).unwrap();
                        if cbor.len() <= buffer_len as usize {
                            let memory_data_mut = memory.data_mut(&mut caller);
                            memory_data_mut[buffer as usize..(buffer as usize + cbor.len())]
                                .copy_from_slice(&cbor);
                        }
                        cbor.len() as u32
                    })
                }
            })
            .unwrap();
        self.linker
            .func_wrap_async("registry", "write", {
                let link = link.clone();
                move |mut caller: Caller<'_, CallState>, params: (u32, u32, u32, u32)| {
                    let link = link.clone();
                    Box::new(async move {
                        let (topic, topic_len, payload, payload_len) = params;
                        let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                            return;
                        };
                        let memory_data = memory.data(&caller);
                        let Some(topic) = memory_data
                            .get(topic as usize..(topic + topic_len) as usize)
                            .and_then(|s| std::str::from_utf8(s).ok())
                        else {
                            return;
                        };
                        let Some(payload) =
                            memory_data.get(payload as usize..(payload + payload_len) as usize)
                        else {
                            return;
                        };
                        let topic = Topic::from_str(topic);
                        let Ok(value) = ciborium::de::from_reader(payload) else {
                            return;
                        };
                        tokio::task::spawn(async move {
                            if let Err(error) = link.registry_write(topic.clone(), value).await {
                                tracing::error!("Failed to write to registry ({topic}): {error}");
                            }
                        });
                    })
                }
            })
            .unwrap();
        self.linker
            .func_wrap_async(
                "registry",
                "delete",
                move |mut caller: Caller<'_, CallState>, params: (u32, u32)| {
                    let link = link.clone();
                    Box::new(async move {
                        let (topic, topic_len) = params;
                        let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                            return;
                        };
                        let memory_data = memory.data(&caller);
                        let Some(topic) = memory_data
                            .get(topic as usize..(topic + topic_len) as usize)
                            .and_then(|s| std::str::from_utf8(s).ok())
                        else {
                            return;
                        };
                        let topic = Topic::from_str(topic);
                        tokio::task::spawn(async move {
                            if let Err(error) = link.registry_delete(topic.clone()).await {
                                tracing::error!(
                                    "Failed to delete from registry ({topic}): {error}"
                                );
                            }
                        });
                    })
                },
            )
            .unwrap();
    }
}

impl Linkage for Arc<RwLock<Core>> {
    async fn registry_read(
        &self,
        topic: Topic,
    ) -> Result<Entry> {
        self.read().await.registry_read(topic).await
    }

    async fn registry_write(
        &self,
        topic: Topic,
        value: Value,
    ) -> Result<()> {
        self.read().await.registry_write(topic, value).await
    }

    async fn registry_delete(
        &self,
        topic: Topic,
    ) -> Result<Entry> {
        self.read().await.registry_delete(topic).await
    }
}
