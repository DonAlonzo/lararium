use crate::Segment;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub segments: Vec<Segment>,
}

impl Topic {
    pub fn from_str(key: &str) -> Self {
        let segments = key.split('/').map(Segment::from_str).collect();
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
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}
