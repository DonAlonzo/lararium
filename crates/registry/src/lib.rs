mod error;
pub use error::*;

use dashmap::{DashMap, DashSet};
use derive_more::{Display, From, Into};
use lararium::{Entry, Filter, Segment, Topic};
use std::hash::Hash;
use std::sync::RwLock;

pub struct Registry {
    root: Node,
}

#[derive(Default)]
struct Node {
    slot: RwLock<Option<Entry>>,
    children: DashMap<Segment, Node>,
    subscriptions: DashSet<u64>,
}

impl Node {
    fn subscribe(
        &self,
        client_id: u64,
        filter: &Filter,
    ) -> Result<()> {
        self.subscriptions.insert(client_id);
        Ok(())
    }

    fn unsubscribe(
        &self,
        client_id: u64,
        filter: &Filter,
    ) -> Result<()> {
        let subscription = self.subscriptions.remove(&client_id);
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
        let segment = segments[0].clone();
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
        let segment = segments[0].clone();
        let node = self.children.get(&segment).ok_or(Error::EntryNotFound)?;
        node.read(&segments[1..])
    }

    fn update(
        &self,
        segments: &[Segment],
        payload: &[u8],
        mut subscribers: DashSet<u64>,
    ) -> Result<(DashSet<u64>, Entry)> {
        subscribers.extend(self.subscriptions.iter().map(|entry| *entry));
        if segments.is_empty() {
            let mut slot = self.slot.write().unwrap();
            return match slot.as_mut() {
                Some(Entry::Signal) => {
                    if payload.is_empty() {
                        Ok((subscribers, Entry::Signal))
                    } else {
                        Err(Error::InvalidPayload)
                    }
                }
                Some(Entry::Boolean(opt_bool)) => {
                    let value = match payload {
                        [0x00] => false,
                        [_] => true,
                        _ => return Err(Error::InvalidPayload),
                    };
                    *opt_bool = value;
                    Ok((subscribers, Entry::Boolean(value)))
                }
                None => Err(Error::EntryNotFound),
            };
        }
        let segment = &segments[0];
        let node = self.children.get(&segment).ok_or(Error::EntryNotFound)?;
        node.update(&segments[1..], payload, subscribers)
    }

    fn delete(
        &self,
        segments: &[Segment],
    ) -> Result<(Vec<u64>, Entry)> {
        if segments.is_empty() {
            let slot = self.slot.write().unwrap().take();
            return Ok((vec![], slot.unwrap()));
        }
        let segment = &segments[0];
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
        client_id: u64,
        filter: &Filter,
    ) -> Result<()> {
        tracing::debug!("[registry::subscribe] {client_id} {filter:?}");
        self.root.subscribe(client_id, filter)?;
        Ok(())
    }

    pub fn unsubscribe(
        &self,
        client_id: u64,
        filter: &Filter,
    ) -> Result<()> {
        tracing::debug!("[registry::unsubscribe] {client_id} {filter:?}");
        self.root.unsubscribe(client_id, filter)
    }

    pub fn create(
        &self,
        topic: &Topic,
        entry: Entry,
    ) -> Result<Vec<u64>> {
        tracing::debug!("[registry::create] {:?}", topic);
        self.root.create(&topic.segments, entry)?;
        Ok(vec![])
    }

    pub fn read(
        &self,
        topic: &Topic,
    ) -> Result<Entry> {
        tracing::debug!("[registry::read] {:?}", topic);
        self.root.read(&topic.segments)
    }

    pub fn update(
        &self,
        topic: &Topic,
        payload: &[u8],
    ) -> Result<(Vec<u64>, Entry)> {
        tracing::debug!("[registry::update] {:?} {:?}", topic, payload);
        let (subscribers, entry) = self.root.update(&topic.segments, payload, DashSet::new())?;
        Ok((subscribers.into_iter().collect(), entry))
    }

    pub fn delete(
        &self,
        topic: &Topic,
    ) -> Result<(Vec<u64>, Entry)> {
        tracing::debug!("[registry::delete] {:?}", topic);
        self.root.delete(&topic.segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        let subscriptions = registry.create(&topic, entry).unwrap();
        assert_eq!(subscriptions.len(), 0);
    }

    #[test]
    fn test_create_conflict() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        registry.create(&topic, entry).unwrap();
        let result = registry.create(&topic, entry);
        assert_eq!(result, Err(Error::Conflict));
    }

    #[test]
    fn test_read_not_found() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let result = registry.read(&topic);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_read() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        registry.create(&topic, entry).unwrap();
        let result = registry.read(&topic);
        assert_eq!(result, Ok(entry));
    }

    #[test]
    fn test_delete_not_found() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let result = registry.delete(&topic);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_delete() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        registry.create(&topic, entry).unwrap();
        let result = registry.delete(&topic);
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
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(true);
        let subscriptions = registry.create(&topic, entry).unwrap();
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0], subscription);
    }

    #[test]
    fn test_update_not_found() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let payload = &[0, 1, 2];
        let result = registry.update(&topic, payload);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_update_bool_set_true() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(false);
        registry.create(&topic, entry).unwrap();
        let payload = &[0x11];
        let result = registry.update(&topic, payload);
        assert_eq!(result, Ok((vec![], Entry::Boolean(true))));
    }

    #[test]
    fn test_update_bool_invalid_payload() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment(0), Segment(1), Segment(2)],
        };
        let entry = Entry::Boolean(false);
        registry.create(&topic, entry).unwrap();
        let payload = &[0x11, 0x11];
        let result = registry.update(&topic, payload);
        assert_eq!(result, Err(Error::InvalidPayload));
    }
}
