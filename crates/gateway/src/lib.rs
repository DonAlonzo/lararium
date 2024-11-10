mod api;
mod dhcp;
mod dns;
mod mqtt;

use lararium_crypto::{Certificate, Identity};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Gateway {
    ca: Certificate,
    identity: Identity,
    inner: Arc<RwLock<Inner>>,
}

struct Inner {
    subscriptions: HashMap<String, Vec<Subscription>>,
}

#[derive(Clone)]
pub struct Subscription {
    tx: flume::Sender<Vec<u8>>,
}

impl Gateway {
    pub fn new(
        ca: Certificate,
        identity: Identity,
    ) -> Self {
        Self {
            ca,
            identity,
            inner: Arc::new(RwLock::new(Inner {
                subscriptions: HashMap::new(),
            })),
        }
    }

    pub async fn get_subscriptions<'a>(
        &'a self,
        topic: &str,
    ) -> Option<Vec<Subscription>> {
        let inner = self.inner.read().await;
        inner.subscriptions.get(topic).cloned()
    }

    pub async fn add_subscription(
        &mut self,
        topic: &str,
        subscription: Subscription,
    ) {
        let mut inner = self.inner.write().await;
        inner
            .subscriptions
            .entry(topic.to_string())
            .or_default()
            .push(subscription);
    }
}
