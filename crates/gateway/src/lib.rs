mod api;
mod dhcp;
mod dns;
mod mqtt;
mod nfs;
mod prelude;

use lararium::prelude::*;
use lararium_crypto::{Certificate, Identity};
use std::sync::Arc;
use wasmtime::*;

#[derive(Clone)]
pub struct Gateway {
    ca: Certificate,
    identity: Identity,
    registry: Arc<lararium_registry::Registry<Subscriber>>,
    mqtt: lararium_mqtt::Server<Gateway>,
    dns: lararium_dns::Server,
    dhcp: lararium_dhcp::Server,
    nfs: lararium_nfs::Server,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Subscriber {
    Client(u64),
    Module(u64),
}

impl Gateway {
    pub async fn new(
        ca: Certificate,
        identity: Identity,
        mqtt: lararium_mqtt::Server<Gateway>,
        dns: lararium_dns::Server,
        dhcp: lararium_dhcp::Server,
        nfs: lararium_nfs::Server,
    ) -> Self {
        let registry = Arc::new(lararium_registry::Registry::new());
        Self {
            ca,
            identity,
            registry,
            mqtt,
            dns,
            dhcp,
            nfs,
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
        Ok(())
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
}
