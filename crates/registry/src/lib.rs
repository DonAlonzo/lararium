mod error;
pub use error::*;

use dashmap::{DashMap, DashSet};
use lararium::prelude::*;
use std::hash::Hash;
use std::sync::RwLock;

pub struct Registry<T>
where
    T: PartialEq + Eq + Hash + Clone,
{
    root: Node<T>,
}

struct Node<T>
where
    T: PartialEq + Eq + Hash + Clone,
{
    slot: RwLock<Option<Entry>>,
    children: DashMap<Segment, Node<T>>,
    subscriptions: DashSet<T>,
}

impl<T> Node<T>
where
    T: PartialEq + Eq + Hash + Clone,
{
    fn new() -> Self {
        Self {
            slot: RwLock::new(None),
            children: DashMap::new(),
            subscriptions: DashSet::new(),
        }
    }

    fn subscribe(
        &self,
        subscriber: T,
        filter: &Filter,
        depth: usize,
    ) -> Result<()> {
        if depth == filter.segments.len() {
            self.subscriptions.insert(subscriber);
            return Ok(());
        }
        let segment = filter.segments[depth].clone().unwrap();
        let child = self.children.entry(segment).or_insert_with(Node::new);
        child.subscribe(subscriber, filter, depth + 1)
    }

    fn unsubscribe(
        &self,
        subscriber: T,
        filter: &Filter,
        depth: usize,
    ) -> Result<()> {
        if depth == filter.segments.len() {
            let subscription = self.subscriptions.remove(&subscriber);
            if subscription.is_none() {
                return Err(Error::SubscriptionNotFound);
            } else {
                return Ok(());
            }
        }
        let segment = filter.segments[depth].clone().unwrap();
        let child = self.children.entry(segment).or_insert_with(Node::new);
        child.unsubscribe(subscriber, filter, depth + 1)
    }

    fn create(
        &self,
        segments: &[Segment],
        entry: Entry,
        mut subscribers: DashSet<T>,
    ) -> Result<DashSet<T>> {
        let mut slot = self.slot.write().unwrap();
        if segments.is_empty() {
            subscribers.extend(self.subscriptions.iter().map(|entry| entry.clone()));
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
        let node = self.children.entry(segment).or_insert_with(Node::new);
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
        new_value: Value,
        mut subscribers: DashSet<T>,
    ) -> Result<(DashSet<T>, Entry)> {
        if segments.is_empty() {
            subscribers.extend(self.subscriptions.iter().map(|entry| entry.clone()));
            let mut slot = self.slot.write().unwrap();
            return match slot.as_mut() {
                None => Err(Error::EntryNotFound),
                Some(Entry::Directory) => Err(Error::InvalidOperation),
                Some(Entry::Signal { schema }) => {
                    if schema.validate(&new_value) {
                        Ok((
                            subscribers,
                            Entry::Signal {
                                schema: schema.clone(),
                            },
                        ))
                    } else {
                        Err(Error::InvalidPayload)
                    }
                }
                Some(Entry::Record { schema, value }) => {
                    if schema.validate(&new_value) {
                        *value = new_value.clone();
                        Ok((
                            subscribers,
                            Entry::Record {
                                schema: schema.clone(),
                                value: new_value,
                            },
                        ))
                    } else {
                        Err(Error::InvalidPayload)
                    }
                }
            };
        }
        let segment = &segments[0];
        let node = self.children.get(segment).ok_or(Error::EntryNotFound)?;
        node.update(&segments[1..], new_value, subscribers)
    }

    fn delete(
        &self,
        segments: &[Segment],
    ) -> Result<(Vec<T>, Entry)> {
        if segments.is_empty() {
            let slot = self.slot.write().unwrap().take();
            return Ok((vec![], slot.unwrap()));
        }
        let segment = &segments[0];
        let node = self.children.get(segment).ok_or(Error::EntryNotFound)?;
        node.delete(&segments[1..])
    }
}

impl<T> Registry<T>
where
    T: PartialEq + Eq + Hash + Clone + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self { root: Node::new() }
    }

    pub fn subscribe(
        &self,
        subscriber: T,
        filter: &Filter,
    ) -> Result<()> {
        tracing::debug!("[registry::subscribe] {subscriber:?} {filter}");
        self.root.subscribe(subscriber, filter, 0)
    }

    pub fn unsubscribe(
        &self,
        subscriber: T,
        filter: &Filter,
    ) -> Result<()> {
        tracing::debug!("[registry::unsubscribe] {subscriber:?} {filter}");
        self.root.unsubscribe(subscriber, filter, 0)
    }

    pub fn create(
        &self,
        topic: &Topic,
        entry: Entry,
    ) -> Result<Vec<T>> {
        tracing::debug!("[registry::create] {topic}");
        if let Entry::Record { schema, value } = &entry {
            if !schema.validate(value) {
                return Err(Error::InvalidPayload);
            }
        }
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
        value: Value,
    ) -> Result<(Vec<T>, Entry)> {
        tracing::debug!("[registry::update] {topic} {value:?}");
        let (subscribers, entry) = self.root.update(&topic.segments, value, DashSet::new())?;
        Ok((subscribers.into_iter().collect(), entry))
    }

    pub fn delete(
        &self,
        topic: &Topic,
    ) -> Result<(Vec<T>, Entry)> {
        tracing::debug!("[registry::delete] {topic}");
        self.root.delete(&topic.segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        let subscriptions = registry.create(&topic, entry).unwrap();
        assert_eq!(subscriptions.len(), 0);
    }

    #[test]
    fn test_create_conflict() {
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        registry.create(&topic, entry.clone()).unwrap();
        let result = registry.create(&topic, entry);
        assert_eq!(result, Err(Error::Conflict));
    }

    #[test]
    fn test_create_conflicting_schema() {
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Record {
            schema: Schema::Boolean,
            value: Value::Text(String::from("hello world")),
        };
        let result = registry.create(&topic, entry);
        assert_eq!(result, Err(Error::InvalidPayload));
    }

    #[test]
    fn test_read_not_found() {
        let registry = Registry::<u64>::new();
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
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        registry.create(&topic, entry.clone()).unwrap();
        let result = registry.read(&topic);
        assert_eq!(result, Ok(entry));
    }

    #[test]
    fn test_delete_not_found() {
        let registry = Registry::<u64>::new();
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
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        registry.create(&topic, entry).unwrap();
        let result = registry.delete(&topic);
        assert_eq!(
            result,
            Ok((
                vec![],
                Entry::Record {
                    schema: Schema::Null,
                    value: Value::Null
                }
            ))
        );
    }

    #[test]
    fn test_subscribe_and_create() {
        let registry = Registry::<u64>::new();
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
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        let subscribers = registry.create(&topic, entry).unwrap();
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], 0);
    }

    #[test]
    fn test_subscribe_and_create_another_topic() {
        let registry = Registry::<u64>::new();
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
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        let subscribers = registry.create(&topic, entry).unwrap();
        assert_eq!(subscribers.len(), 0);
    }

    #[test]
    fn test_update_not_found() {
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let result = registry.update(&topic, Value::Null);
        assert_eq!(result, Err(Error::EntryNotFound));
    }

    #[test]
    fn test_create_check_directories() {
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![
                Segment::from_str("0"),
                Segment::from_str("1"),
                Segment::from_str("2"),
            ],
        };
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        registry.create(&topic, entry).unwrap();
        let parent = registry.read(&topic.parent()).unwrap();
        let parent_parent = registry.read(&topic.parent().parent()).unwrap();
        assert_eq!(parent, Entry::Directory);
        assert_eq!(parent_parent, Entry::Directory);
    }

    #[test]
    fn test_create_parent_not_directory() {
        let registry = Registry::<u64>::new();
        let topic = Topic {
            segments: vec![Segment::from_str("0"), Segment::from_str("1")],
        };
        let entry = Entry::Record {
            schema: Schema::Null,
            value: Value::Null,
        };
        registry.create(&topic, entry.clone()).unwrap();
        let result = registry.create(&topic.child(Segment::from_str("3")), entry);
        assert_eq!(result, Err(Error::Conflict));
    }
}
