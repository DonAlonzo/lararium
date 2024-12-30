use crate::Segment;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub segments: Vec<Segment>,
}

impl From<&str> for Topic {
    fn from(key: &str) -> Self {
        let segments = key.split('/').map(Segment::from).collect();
        Self { segments }
    }
}

impl From<String> for Topic {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Topic {
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
                .map(Segment::as_ref)
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}
