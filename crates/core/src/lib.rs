pub mod prelude;
pub mod registry;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Entry {
    Signal,
    Boolean(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Filter {
    pub segments: Vec<Option<Segment>>,
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    pub segments: Vec<Segment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Segment(u64);

impl Filter {
    pub fn from_str(filter: &str) -> Self {
        let segments = filter.split('/').map(Segment::from_str).map(Some).collect();
        Self {
            segments,
            open: false,
        }
    }
}

impl Key {
    pub fn from_str(key: &str) -> Self {
        let segments = key.split('/').map(Segment::from_str).collect();
        Self { segments }
    }
}

impl Segment {
    pub fn from_str(segment: &str) -> Self {
        Self(0)
    }
}
