use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Segment(String);

impl From<&str> for Segment {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Segment {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for Segment {
    fn as_ref(&self) -> &str {
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
