use crate::{Segment, Topic};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    pub segments: Vec<Option<Segment>>,
    pub open: bool,
}

impl Filter {
    pub fn from_str(filter: &str) -> Self {
        let segments = filter.split('/').map(Segment::from_str).map(Some).collect();
        Self {
            segments,
            open: false,
        }
    }
}

impl From<Topic> for Filter {
    fn from(topic: Topic) -> Self {
        Filter {
            segments: topic.segments.into_iter().map(Some).collect(),
            open: false,
        }
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
                .map(|s| s.as_ref().map(Segment::as_str).unwrap_or("*"))
                .collect::<Vec<_>>()
                .join("/")
        )
    }
}
