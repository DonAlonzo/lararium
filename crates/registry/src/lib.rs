mod error;
pub use error::*;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Signal,
    Boolean(bool),
}

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

    pub async fn create(
        &self,
        topic_name: &str,
        entry: Entry,
    ) -> Result<()> {
        tracing::debug!("[registry::create] {}", topic_name);
        let mut entries = self.entries.write().await;
        if entries.contains_key(topic_name) {
            Err(Error::Conflict)
        } else {
            entries.insert(topic_name.to_string(), entry);
            Ok(())
        }
    }

    pub async fn read(
        &self,
        topic_name: &str,
    ) -> Result<Entry> {
        tracing::debug!("[registry::read] {}", topic_name);
        let entries = self.entries.read().await;
        entries.get(topic_name).cloned().ok_or(Error::EntryNotFound)
    }

    pub async fn write(
        &self,
        topic_name: &str,
        payload: &[u8],
    ) -> Result<()> {
        tracing::debug!("[registry::write] {} {:?}", topic_name, payload);
        let mut entries = self.entries.write().await;
        match entries.get_mut(topic_name) {
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
        topic_name: &str,
    ) -> Result<Entry> {
        tracing::debug!("[registry::delete] {}", topic_name);
        let mut entries = self.entries.write().await;
        entries.remove(topic_name).ok_or(Error::EntryNotFound)
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
}
