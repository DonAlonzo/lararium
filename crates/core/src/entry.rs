use crate::{Schema, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Entry {
    Directory,
    Signal { schema: Schema },
    Record { schema: Schema, value: Value },
}
