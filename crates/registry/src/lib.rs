mod error;
pub use error::*;

use dashmap::{DashMap, DashSet};
use lararium::prelude::*;
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
        depth: usize,
    ) -> Result<()> {
        if depth == filter.segments.len() {
            self.subscriptions.insert(client_id);
            return Ok(());
        }
        let segment = filter.segments[depth].clone().unwrap();
        let child = self.children.entry(segment).or_default();
        child.subscribe(client_id, filter, depth + 1)
    }

    fn unsubscribe(
        &self,
        client_id: u64,
        filter: &Filter,
        depth: usize,
    ) -> Result<()> {
        if depth == filter.segments.len() {
            let subscription = self.subscriptions.remove(&client_id);
            if subscription.is_none() {
                return Err(Error::SubscriptionNotFound);
            } else {
                return Ok(());
            }
        }
        let segment = filter.segments[depth].clone().unwrap();
        let child = self.children.entry(segment).or_default();
        child.unsubscribe(client_id, filter, depth + 1)
    }

    fn create(
        &self,
        segments: &[Segment],
        entry: Entry,
        mut subscribers: DashSet<u64>,
    ) -> Result<DashSet<u64>> {
        let mut slot = self.slot.write().unwrap();
        if segments.is_empty() {
            subscribers.extend(self.subscriptions.iter().map(|entry| *entry));
            if slot.is_some() {
                return Err(Error::Conflict);
            }
            *slot = Some(entry);
            return Ok(subscribers);
        }
        match slot.as_ref() {
            Some(Entry::Directory) => (),
            Some(_) => return Err(Error::Conflict),
            None => *slot = Some(Entry::Directory),
        };
        let segment = segments[0].clone();
        let node = self.children.entry(segment).or_default();
        node.create(&segments[1..], entry, subscribers)
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
        if segments.is_empty() {
            subscribers.extend(self.subscriptions.iter().map(|entry| *entry));
            let mut slot = self.slot.write().unwrap();
            return match slot.as_mut() {
                None => Err(Error::EntryNotFound),
                Some(Entry::Directory) => Err(Error::InvalidOperation),
                Some(Entry::Signal) => Ok((subscribers, Entry::Signal)),
                Some(Entry::Boolean(opt_bool)) => {
                    let value = match payload {
                        [0x00] => false,
                        [_] => true,
                        _ => return Err(Error::InvalidPayload),
                    };
                    *opt_bool = value;
                    Ok((subscribers, Entry::Boolean(value)))
                }
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
        tracing::debug!("[registry::subscribe] {client_id} {filter}");
        self.root.subscribe(client_id, filter, 0)
    }

    pub fn unsubscribe(
        &self,
        client_id: u64,
        filter: &Filter,
    ) -> Result<()> {
        tracing::debug!("[registry::unsubscribe] {client_id} {filter}");
        self.root.unsubscribe(client_id, filter, 0)
    }

    pub fn create(
        &self,
        topic: &Topic,
        entry: Entry,
    ) -> Result<Vec<u64>> {
        tracing::debug!("[registry::create] {topic}");
        let subscribers = self.root.create(&topic.segments, entry, DashSet::new())?;
        Ok(subscribers.into_iter().collect())
    }

    pub fn read(
        &self,
        topic: &Topic,
    ) -> Result<Entry> {
        tracing::debug!("[registry::read] {topic}");
        self.root.read(&topic.segments)
    }

    pub fn update(
        &self,
        topic: &Topic,
        payload: &[u8],
    ) -> Result<(Vec<u64>, Entry)> {
        tracing::debug!("[registry::update] {topic} {payload:?}");
        let (subscribers, entry) = self.root.update(&topic.segments, payload, DashSet::new())?;
        Ok((subscribers.into_iter().collect(), entry))
    }

    pub fn delete(
        &self,
        topic: &Topic,
    ) -> Result<(Vec<u64>, Entry)> {
        tracing::debug!("[registry::delete] {topic}");
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
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Boolean(true);
        let subscriptions = registry.create(&topic, entry).unwrap();
        assert_eq!(subscriptions.len(), 0);
    }

    #[test]
    fn test_create_conflict() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
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
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let result = registry.read(&topic);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_read() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
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
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let result = registry.delete(&topic);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_delete() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Boolean(true);
        registry.create(&topic, entry).unwrap();
        let result = registry.delete(&topic);
        assert_eq!(result, Ok((vec![], Entry::Boolean(true))));
    }

    #[test]
    fn test_subscribe_and_create() {
        let registry = Registry::new();
        let filter = Filter {
            segments: vec![
                Some(Segment::from_str("0")),
                Some(Segment::from_str("1")),
                Some(Segment::from_str("2")),
            ],
            open: false,
        };
        registry.subscribe(0, &filter).unwrap();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Boolean(true);
        let client_ids = registry.create(&topic, entry).unwrap();
        assert_eq!(client_ids.len(), 1);
        assert_eq!(client_ids[0], 0);
    }

    #[test]
    fn test_subscribe_and_create_another_topic() {
        let registry = Registry::new();
        let filter = Filter {
            segments: vec![Some(Segment::from_str("0")), Some(Segment::from_str("1"))],
            open: false,
        };
        registry.subscribe(0, &filter).unwrap();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("2"),
                Segment::from_str("1"),
            ],
        };
        let entry = Entry::Boolean(true);
        let client_ids = registry.create(&topic, entry).unwrap();
        assert_eq!(client_ids.len(), 0);
    }

    #[test]
    fn test_update_not_found() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let payload = &[0, 1, 2];
        let result = registry.update(&topic, payload);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_update_bool_set_true() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
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
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Boolean(false);
        registry.create(&topic, entry).unwrap();
        let payload = &[0x11, 0x11];
        let result = registry.update(&topic, payload);
        assert_eq!(result, Err(Error::InvalidPayload));
    }

    #[test]
    fn test_create_check_directories() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Boolean(true);
        registry.create(&topic, entry).unwrap();
        let parent = registry.read(&topic.parent()).unwrap();
        let parent_parent = registry.read(&topic.parent().parent()).unwrap();
        assert_eq!(parent, Entry::Directory);
        assert_eq!(parent_parent, Entry::Directory);
    }

    #[test]
    fn test_create_parent_not_directory() {
        let registry = Registry::new();
        let topic = Topic {
            segments: vec![Segment::from_str("0"), Segment::from_str("1")],
        };
        let entry = Entry::Boolean(true);
        registry.create(&topic, entry).unwrap();
        let result = registry.create(&topic.child(Segment::from_str("3")), entry);
        assert_eq!(result, Err(Error::Conflict));
    }
}
