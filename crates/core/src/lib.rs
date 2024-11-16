pub mod prelude;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

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

impl Key {
    pub fn from_str(key: &str) -> Self {
        let segments = key.split('/').map(String::from).map(Segment).collect();
        Self { segments }
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

impl Display for Key {
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
