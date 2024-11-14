mod error;
pub use error::*;

use dashmap::DashMap;
use derive_more::{Display, From, Into};
use std::hash::Hash;
use std::sync::RwLock;

pub struct Registry {
    root: Node,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, From, Into, Display)]
pub struct Subscription(u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    segments: Vec<Option<Segment>>,
    open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    segments: Vec<Segment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segment(u64);

#[derive(Default)]
struct Node {
    slot: RwLock<Option<Entry>>,
    children: DashMap<Segment, Node>,
    subscriptions: DashMap<Subscription, Filter>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Entry {
    Signal,
    Boolean(bool),
}

impl Node {
    fn subscribe(
        &self,
        subscription: Subscription,
        filter: &Filter,
    ) -> Result<()> {
        self.subscriptions.insert(subscription, filter.clone());
        Ok(())
    }

    fn unsubscribe(
        &self,
        subscription: Subscription,
    ) -> Result<()> {
        let subscription = self.subscriptions.remove(&subscription);
        if subscription.is_none() {
            Err(Error::SubscriptionNotFound)
        } else {
            Ok(())
        }
    }

    fn create(
        &self,
        segments: &[Segment],
        entry: Entry,
    ) -> Result<()> {
        if segments.is_empty() {
            let mut slot = self.slot.write().unwrap();
            if slot.is_some() {
                return Err(Error::Conflict);
            }
            *slot = Some(entry);
            return Ok(());
        }
        let segment = segments[0];
        let node = self.children.entry(segment).or_default();
        node.create(&segments[1..], entry)
    }

    fn read(
        &self,
        segments: &[Segment],
    ) -> Result<Entry> {
        if segments.is_empty() {
            let slot = self.slot.read().unwrap();
            return slot.clone().ok_or(Error::EntryNotFound);
        }
        let segment = segments[0];
        let node = self.children.get(&segment).ok_or(Error::EntryNotFound)?;
        node.read(&segments[1..])
    }

    fn update(
        &self,
        segments: &[Segment],
        payload: &[u8],
    ) -> Result<(Vec<Subscription>, Entry)> {
        if segments.is_empty() {
            let slot = self.slot.write().unwrap();
            return match *slot {
                Some(mut slot) => match slot {
                    Entry::Signal => {
                        if payload.is_empty() {
                            Ok((vec![], Entry::Signal))
                        } else {
                            Err(Error::InvalidPayload)
                        }
                    }
                    Entry::Boolean(ref mut opt_bool) => {
                        let value = match payload {
                            [b'f', b'a', b'l', b's', b'e'] => false,
                            [b't', b'r', b'u', b'e'] => true,
                            _ => return Err(Error::InvalidPayload),
                        };
                        *opt_bool = value;
                        Ok((vec![], Entry::Boolean(value)))
                    }
                },
                None => Err(Error::EntryNotFound),
            };
        }
        let segment = segments[0];
        let node = self.children.get(&segment).ok_or(Error::EntryNotFound)?;
        node.update(&segments[1..], payload)
    }

    fn delete(
        &self,
        segments: &[Segment],
    ) -> Result<(Vec<Subscription>, Entry)> {
        if segments.is_empty() {
            let slot = self.slot.write().unwrap().take();
            return Ok((vec![], slot.unwrap()));
        }
        let segment = segments[0];
        let node = self.children.get(&segment).ok_or(Error::EntryNotFound)?;
        node.delete(&segments[1..])
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            root: Node::default(),
        }
    }

    pub fn subscribe(
        &self,
        filter: &Filter,
    ) -> Result<Subscription> {
        tracing::debug!("[registry::subscribe] {:?}", filter);
        let subscription = Subscription(0);
        self.root.subscribe(subscription, filter)?;
        Ok(subscription)
    }

    pub fn unsubscribe(
        &self,
        subscription: Subscription,
    ) -> Result<()> {
        tracing::debug!("[registry::unsubscribe] {}", subscription);
        self.root.unsubscribe(subscription)
    }

    pub fn create(
        &self,
        key: &Key,
        entry: Entry,
    ) -> Result<Vec<Subscription>> {
        tracing::debug!("[registry::create] {:?}", key);
        self.root.create(&key.segments, entry)?;
        Ok(vec![])
    }

    pub fn read(
        &self,
        key: &Key,
    ) -> Result<Entry> {
        tracing::debug!("[registry::read] {:?}", key);
        self.root.read(&key.segments)
    }

    pub fn update(
        &self,
        key: &Key,
        payload: &[u8],
    ) -> Result<(Vec<Subscription>, Entry)> {
        tracing::debug!("[registry::update] {:?} {:?}", key, payload);
        self.root.update(&key.segments, payload)
    }

    pub fn delete(
        &self,
        key: &Key,
    ) -> Result<(Vec<Subscription>, Entry)> {
        tracing::debug!("[registry::delete] {:?}", key);
        self.root.delete(&key.segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        let subscriptions = registry.create(&key, entry).unwrap();
        assert_eq!(subscriptions.len(), 0);
    }

    #[test]
    fn test_create_conflict() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        registry.create(&key, entry).unwrap();
        let result = registry.create(&key, entry);
        assert_eq!(result, Err(Error::Conflict));
    }

    #[test]
    fn test_read_not_found() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let result = registry.read(&key);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_read() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        registry.create(&key, entry).unwrap();
        let result = registry.read(&key);
        assert_eq!(result, Ok(entry));
    }

    #[test]
    fn test_delete_not_found() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let result = registry.delete(&key);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_delete() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        registry.create(&key, entry).unwrap();
        let result = registry.delete(&key);
        assert_eq!(result, Ok((vec![], Entry::Boolean(true))));
    }

    #[test]
    fn test_subscribe() {
        let registry = Registry::new();
        let filter = Filter {
            segments: vec![Some(Segment(0)), Some(Segment(1)), None],
            open: false,
        };
        let subscription = registry.subscribe(&filter).unwrap();
        assert_eq!(subscription, Subscription(0));
    }

    #[test]
    fn test_subscribe_and_unsubscribe() {
        let registry = Registry::new();
        let filter = Filter {
            segments: vec![Some(Segment(0)), Some(Segment(1)), None],
            open: false,
        };
        let subscription = registry.subscribe(&filter).unwrap();
        let result = registry.unsubscribe(subscription);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_unsubscribe_not_found() {
        let registry = Registry::new();
        let subscription = Subscription(0);
        let result = registry.unsubscribe(subscription);
        assert_eq!(result, Err(Error::SubscriptionNotFound));
    }

    #[test]
    fn test_subscribe_and_create() {
        let registry = Registry::new();
        let filter = Filter {
            segments: vec![Some(Segment(0)), Some(Segment(1)), None],
            open: false,
        };
        let subscription = registry.subscribe(&filter).unwrap();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        let subscriptions = registry.create(&key, entry).unwrap();
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0], subscription);
    }

    #[test]
    fn test_update_not_found() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let payload = &[0, 1, 2];
        let result = registry.update(&key, payload);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_update_bool_set_true() {
        let registry = Registry::new();
        let key = Key {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(false);
        registry.create(&key, entry).unwrap();
        let payload = &[b't', b'r', b'u', b'e'];
        let result = registry.update(&key, payload);
        assert_eq!(result, Ok((vec![], Entry::Boolean(true))));
    }
}
