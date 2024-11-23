pub mod prelude;

use derive_more::{From, Into};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, From, Into, Serialize, Deserialize)]
pub struct Cbor(Vec<u8>);

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i128),
    Bytes(Vec<u8>),
    Float(f64),
    Text(String),
    Boolean(bool),
    Null,
    Tag(u64, Box<Value>),
    Array(Vec<Value>),
    Map(Vec<(String, Box<Value>)>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Schema {
    Integer,
    Bytes,
    Float,
    Text,
    Boolean,
    Optional(Box<Schema>),
    Tag(u64, Box<Schema>),
    Array(Box<Schema>),
    Map(Vec<(String, Box<Schema>)>),
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
        value: &Value,
    ) -> bool {
        match (self, value) {
            (Schema::Integer, Value::Integer(_)) => true,
            (Schema::Bytes, Value::Bytes(_)) => true,
            (Schema::Float, Value::Float(_)) => true,
            (Schema::Text, Value::Text(_)) => true,
            (Schema::Boolean, Value::Boolean(_)) => true,
            (Schema::Optional(_), Value::Null) => true,
            (Schema::Optional(schema), value) => schema.validate(value),
            (Schema::Tag(expected, schema), value) => {
                if let Value::Tag(actual, value) = value {
                    expected == actual && schema.validate(&value)
                } else {
                    false
                }
            }
            (Schema::Array(schema), Value::Array(items)) => {
                items.iter().all(|item| schema.validate(item))
            }
            (Schema::Map(schemas), Value::Map(values)) => {
                let schemas: HashMap<_, _> = schemas.iter().cloned().collect();
                let values: HashMap<_, _> = values.iter().cloned().collect();
                let all_keys_valid = values.iter().all(|(key, value)| {
                    let schema = schemas.get(key);
                    schema.map_or(false, |schema| schema.validate(value))
                });
                let all_required_present = schemas.iter().all(|(key, schema)| {
                    values.contains_key(key) || matches!(**schema, Schema::Optional(_))
                });
                all_keys_valid && all_required_present
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
        let value = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let valid = schema.validate(&value);
        assert!(valid);
    }

    #[test]
    fn test_schema_array_integers_invalid() {
        let schema = Schema::Array(Box::new(Schema::Integer));
        let value = Value::Text("hello world".to_string());
        let valid = schema.validate(&value);
        assert!(!valid);
    }

    #[test]
    fn test_schema_map() {
        let schema = Schema::Map(vec![
            ("one".into(), Box::new(Schema::Integer)),
            ("two".into(), Box::new(Schema::Integer)),
            ("three".into(), Box::new(Schema::Float)),
        ]);
        let value = Value::Map(vec![
            ("one".to_string(), Box::new(Value::Integer(1))),
            ("two".to_string(), Box::new(Value::Integer(2))),
            ("three".to_string(), Box::new(Value::Float(3.0))),
        ]);
        let valid = schema.validate(&value);
        assert!(valid);
    }

    #[test]
    fn test_schema_map_optional() {
        let schema = Schema::Map(vec![
            ("one".into(), Box::new(Schema::Integer)),
            ("two".into(), Box::new(Schema::Integer)),
            ("three".into(), Box::new(Schema::Float)),
            (
                "four".to_string(),
                Box::new(Schema::Optional(Box::new(Schema::Text))),
            ),
        ]);
        let value = Value::Map(vec![
            ("one".to_string(), Box::new(Value::Integer(1))),
            ("two".to_string(), Box::new(Value::Integer(2))),
            ("three".to_string(), Box::new(Value::Float(3.0))),
        ]);
        let valid = schema.validate(&value);
        assert!(valid);
    }

    #[test]
    fn test_schema_map_extra() {
        let schema = Schema::Map(vec![
            ("one".into(), Box::new(Schema::Integer)),
            ("two".into(), Box::new(Schema::Integer)),
        ]);
        let value = Value::Map(vec![
            ("one".to_string(), Box::new(Value::Integer(1))),
            ("two".to_string(), Box::new(Value::Integer(2))),
            ("three".to_string(), Box::new(Value::Float(3.0))),
        ]);
        let valid = schema.validate(&value);
        assert!(!valid);
    }

    #[test]
    fn test_schema_text() {
        let schema = Schema::Text;
        let value = Value::Text("hello world".to_string());
        let valid = schema.validate(&value);
        assert!(valid);
    }

    #[test]
    fn test_schema_optional_text_null() {
        let schema = Schema::Optional(Box::new(Schema::Text));
        let value = Value::Null;
        let valid = schema.validate(&value);
        assert!(valid);
    }

    #[test]
    fn test_schema_optional_text() {
        let schema = Schema::Optional(Box::new(Schema::Text));
        let value = Value::Text("hello world".to_string());
        let valid = schema.validate(&value);
        assert!(valid);
    }
}
