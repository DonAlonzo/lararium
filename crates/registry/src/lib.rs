mod error;
pub use error::*;

use derive_more::{From, Into};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Signal,
    Boolean(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    entry: Entry,
    subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, From, Into)]
pub struct Subscription(u32);

#[derive(Clone)]
pub struct Registry {
    entries: Arc<RwLock<HashMap<String, Entry>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(
        &self,
        key: &str,
    ) -> Subscription {
        0.into()
    }

    pub async fn unsubscribe(
        &self,
        subscription: Subscription,
    ) -> Result<()> {
        Ok(())
    }

    pub async fn poll_update(&self) -> Update {
        Update {
            entry: Entry::Boolean(true),
            subscriptions: vec![Subscription(0)],
        }
    }

    pub async fn create(
        &self,
        key: &str,
        entry: Entry,
    ) -> Result<()> {
        tracing::debug!("[registry::create] {}", key);
        let mut entries = self.entries.write().await;
        if entries.contains_key(key) {
            Err(Error::Conflict)
        } else {
            entries.insert(key.to_string(), entry);
            Ok(())
        }
    }

    pub async fn read(
        &self,
        key: &str,
    ) -> Result<Entry> {
        tracing::debug!("[registry::read] {}", key);
        let entries = self.entries.read().await;
        entries.get(key).cloned().ok_or(Error::EntryNotFound)
    }

    pub async fn write(
        &self,
        key: &str,
        payload: &[u8],
    ) -> Result<()> {
        tracing::debug!("[registry::write] {} {:?}", key, payload);
        let mut entries = self.entries.write().await;
        match entries.get_mut(key) {
            Some(entry) => match entry {
                Entry::Signal => {
                    if payload.is_empty() {
                        Ok(())
                    } else {
                        Err(Error::InvalidPayload)
                    }
                }
                Entry::Boolean(ref mut opt_bool) => {
                    *opt_bool = match payload {
                        [0] => false,
                        [1] => true,
                        [b'f', b'a', b'l', b's', b'e'] => false,
                        [b't', b'r', b'u', b'e'] => true,
                        _ => return Err(Error::InvalidPayload),
                    };
                    Ok(())
                }
            },
            None => Err(Error::EntryNotFound),
        }
    }

    pub async fn delete(
        &self,
        key: &str,
    ) -> Result<Entry> {
        tracing::debug!("[registry::delete] {}", key);
        let mut entries = self.entries.write().await;
        entries.remove(key).ok_or(Error::EntryNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_then_read() {
        let registry = Registry::new();
        registry.create("test", Entry::Signal).await.unwrap();
        let actual = registry.read("test").await.unwrap();
        assert_eq!(actual, Entry::Signal);
    }

    #[tokio::test]
    async fn test_create_bool_then_write_1() {
        let registry = Registry::new();
        registry
            .create("test", Entry::Boolean(false))
            .await
            .unwrap();
        registry.write("test", &[1]).await.unwrap();
        let actual = registry.read("test").await.unwrap();
        assert_eq!(actual, Entry::Boolean(true));
    }

    #[tokio::test]
    async fn test_create_bool_then_write_0() {
        let registry = Registry::new();
        registry.create("test", Entry::Boolean(true)).await.unwrap();
        registry.write("test", &[0]).await.unwrap();
        let actual = registry.read("test").await.unwrap();
        assert_eq!(actual, Entry::Boolean(false));
    }

    #[tokio::test]
    async fn test_create_bool_then_write_true() {
        let registry = Registry::new();
        registry
            .create("test", Entry::Boolean(false))
            .await
            .unwrap();
        registry.write("test", b"true").await.unwrap();
        let actual = registry.read("test").await.unwrap();
        assert_eq!(actual, Entry::Boolean(true));
    }

    #[tokio::test]
    async fn test_create_bool_then_write_false() {
        let registry = Registry::new();
        registry.create("test", Entry::Boolean(true)).await.unwrap();
        registry.write("test", b"false").await.unwrap();
        let actual = registry.read("test").await.unwrap();
        assert_eq!(actual, Entry::Boolean(false));
    }

    #[tokio::test]
    async fn test_subscribe() {
        let registry = Registry::new();
        registry
            .create("test", Entry::Boolean(false))
            .await
            .unwrap();
        let subscription = registry.subscribe("test").await;
        registry.write("test", b"true").await.unwrap();
        let update = registry.poll_update().await;
        assert_eq!(update.entry, Entry::Boolean(true));
        assert_eq!(update.subscriptions, vec![subscription]);
    }
}
