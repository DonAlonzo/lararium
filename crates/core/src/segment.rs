use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Segment(String);

impl Segment {
    pub fn from_str(segment: &str) -> Self {
        Self(String::from(segment))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
