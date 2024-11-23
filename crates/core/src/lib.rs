pub mod prelude;

use ciborium::Value;
use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, From, Into, Serialize, Deserialize)]
pub struct Cbor(Vec<u8>);

#[derive(Clone, Debug, PartialEq)]
pub enum Schema {
    Integer,
    Bytes,
    Float,
    Text,
    Bool,
    Null,
    Tag(u64, Box<Schema>),
    Array(Box<Schema>),
    Map(Box<Schema>, Box<Schema>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Entry {
    Directory,
    Signal,
    Record(Cbor),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    pub segments: Vec<Option<Segment>>,
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub segments: Vec<Segment>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Segment(String);

impl Schema {
    pub fn validate(
        &self,
        cbor: &Cbor,
    ) -> bool {
        let parsed = ciborium::de::from_reader(cbor.as_bytes()).unwrap();
        self.validate_value(&parsed)
    }

    fn validate_value(
        &self,
        value: &Value,
    ) -> bool {
        match (self, value) {
            (Schema::Integer, Value::Integer(_)) => true,
            (Schema::Bytes, Value::Bytes(_)) => true,
            (Schema::Float, Value::Float(_)) => true,
            (Schema::Text, Value::Text(_)) => true,
            (Schema::Bool, Value::Bool(_)) => true,
            (Schema::Null, Value::Null) => true,
            (Schema::Tag(expected, schema), value) => {
                if let Value::Tag(actual, value) = value {
                    expected == actual && schema.validate_value(&value)
                } else {
                    false
                }
            }
            (Schema::Array(item_schema), Value::Array(items)) => {
                items.iter().all(|item| item_schema.validate_value(item))
            }
            (Schema::Map(key_schema, value_schema), Value::Map(map)) => {
                map.iter().all(|(key, value)| {
                    key_schema.validate_value(key) && value_schema.validate_value(value)
                })
            }
            _ => false,
        }
    }
}

impl Cbor {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

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

impl Into<Filter> for Topic {
    fn into(self) -> Filter {
        Filter {
            segments: self.segments.into_iter().map(Some).collect(),
            open: false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_array_integers() {
        let schema = Schema::Array(Box::new(Schema::Integer));
        let cbor = Cbor(vec![0x83, 0x01, 0x02, 0x03]);
        let valid = schema.validate(&cbor);
        assert!(valid);
    }

    #[test]
    fn test_schema_array_integers_invalid() {
        let schema = Schema::Array(Box::new(Schema::Integer));
        let cbor = Cbor(vec![
            0x6B, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
        ]);
        let valid = schema.validate(&cbor);
        assert!(!valid);
    }

    #[test]
    fn test_schema_map() {
        let schema = Schema::Map(Box::new(Schema::Text), Box::new(Schema::Integer));
        let cbor = Cbor(vec![
            0xA2, 0x63, 0x66, 0x6F, 0x6F, 0x18, 0x64, 0x63, 0x62, 0x61, 0x72, 0x18, 0x64,
        ]);
        let valid = schema.validate(&cbor);
        assert!(valid);
    }

    #[test]
    fn test_schema_text() {
        let schema = Schema::Text;
        let cbor = Cbor(vec![
            0x6B, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
        ]);
        let valid = schema.validate(&cbor);
        assert!(valid);
    }
}
