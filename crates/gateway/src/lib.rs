mod api;
mod dhcp;
mod dns;
mod mqtt;

use lararium::prelude::*;
use lararium_crypto::{Certificate, Identity};
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::*;

#[derive(Clone)]
pub struct Gateway {
    core: Arc<RwLock<Core>>,
}

#[derive(Clone)]
struct Core {
    ca: Certificate,
    identity: Identity,
    engine: Engine,
    linker: Linker<CallState>,
    modules: Vec<Module>,
    registry: Arc<lararium_registry::Registry>,
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
        let wasm =
            std::fs::read("target/wasm32-unknown-unknown/release/lararium_rules.wasm").unwrap();
        core.write().await.add_module(&wasm);
        Self { core }
    }
}

trait Linkage {
    fn registry_write(
        &self,
        topic_name: String,
        payload: Vec<u8>,
    ) -> impl std::future::Future<Output = ()> + Send;
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

        registry
            .create(&Key::from_str("0000/status"), Entry::Boolean(false))
            .unwrap();

        registry
            .create(&Key::from_str("0000/command/play"), Entry::Signal)
            .unwrap();

        Self {
            ca,
            identity,
            engine,
            linker,
            modules: vec![],
            registry,
            mqtt,
            dns,
            dhcp,
        }
    }

    pub fn add_module(
        &mut self,
        wasm: &[u8],
    ) {
        let module = Module::new(&self.engine, &wasm).unwrap();
        self.modules.push(module);
    }

    pub async fn registry_write(
        &self,
        topic_name: &str,
        payload: &[u8],
    ) {
        let key = Key::from_str(topic_name);
        let (client_ids, _) = self.registry.update(&key, payload).unwrap();
        self.mqtt
            .publish(&client_ids, topic_name, payload)
            .await
            .unwrap();
        self.on_registry_write(topic_name.to_string(), payload.to_vec())
            .await;
    }

    async fn on_registry_write(
        &self,
        topic_name: String,
        payload: Vec<u8>,
    ) {
        for module in &self.modules {
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
            let topic_name = topic_name.clone();
            let payload = payload.clone();
            let topic_name_ptr = write_to_memory(topic_name.as_bytes(), &memory, &mut store);
            let payload_ptr = write_to_memory(&payload, &memory, &mut store);
            tokio::task::spawn(async move {
                run.call_async(
                    &mut store,
                    (
                        topic_name_ptr,
                        topic_name.len() as u32,
                        payload_ptr,
                        payload.len() as u32,
                    ),
                )
                .await
                .unwrap();
            });
        }
    }

    pub fn link<T>(
        &mut self,
        link: T,
    ) where
        T: Linkage + Clone + Send + Sync + 'static,
    {
        self.linker
            .func_wrap_async(
                "registry",
                "write",
                move |mut caller: Caller<'_, CallState>, params: (u32, u32, u32, u32)| {
                    let link = link.clone();
                    Box::new(async move {
                        let (topic_name, topic_name_len, payload, payload_len) = params;
                        let Some(Extern::Memory(memory)) = caller.get_export("memory") else {
                            return;
                        };
                        let Some(topic_name) = memory
                            .data(&caller)
                            .get(topic_name as usize..(topic_name + topic_name_len) as usize)
                            .and_then(|s| std::str::from_utf8(s).ok())
                        else {
                            return;
                        };
                        let Some(payload) = memory
                            .data(&caller)
                            .get(payload as usize..(payload + payload_len) as usize)
                        else {
                            return;
                        };
                        let topic_name = topic_name.to_string();
                        let payload = payload.to_vec();
                        tokio::task::spawn(async move {
                            link.registry_write(topic_name, payload).await;
                        });
                    })
                },
            )
            .unwrap();
    }
}

impl Linkage for Arc<RwLock<Core>> {
    async fn registry_write(
        &self,
        topic_name: String,
        payload: Vec<u8>,
    ) {
        self.read()
            .await
            .registry_write(&topic_name, &payload)
            .await;
    }
}
