pub mod prelude;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Entry {
    Directory,
    Signal,
    Cbor(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Filter {
    pub segments: Vec<Option<Segment>>,
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Topic {
    pub segments: Vec<Segment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Segment(String);

impl Filter {
    pub fn from_str(filter: &str) -> Self {
        let segments = filter
            .split('/')
            .map(String::from)
            .map(Segment)
            .map(Some)
            .collect();
        Self {
            segments,
            open: false,
        }
    }
}

impl Topic {
    pub fn from_str(key: &str) -> Self {
        let segments = key.split('/').map(String::from).map(Segment).collect();
        Self { segments }
    }

    pub fn parent(&self) -> Self {
        let segments = self
            .segments
            .iter()
            .take(self.segments.len() - 1)
            .cloned()
            .collect();
        Self { segments }
    }

    pub fn child(
        &self,
        segment: Segment,
    ) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment);
        Self { segments }
    }
}

impl Segment {
    pub fn from_str(segment: &str) -> Self {
        Self(String::from(segment))
    }
}

impl Display for Filter {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .map(|s| s.as_ref().map(|s| s.0.as_str()).unwrap_or("*"))
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}

impl Display for Topic {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .map(|s| s.0.as_str())
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}

impl Display for Segment {
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
