use crate::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Schema {
    Any,
    Integer,
    Bytes,
    Float,
    Text,
    Boolean,
    Null,
    Optional { content: Box<Schema> },
    Tag { tag: u64, content: Box<Schema> },
    Array { content: Box<Schema> },
    Map { content: Vec<(String, Box<Schema>)> },
}

impl Schema {
    pub fn validate(
        &self,
        value: &Value,
    ) -> bool {
        match (self, value) {
            (Schema::Any, _) => true,
            (Schema::Integer, Value::Integer(_)) => true,
            (Schema::Bytes, Value::Bytes(_)) => true,
            (Schema::Float, Value::Float(_)) => true,
            (Schema::Text, Value::Text(_)) => true,
            (Schema::Boolean, Value::Boolean(_)) => true,
            (Schema::Optional { .. }, Value::Null) => true,
            (Schema::Optional { content: schema }, value) => schema.validate(value),
            (Schema::Null, Value::Null) => true,
            (
                Schema::Tag {
                    tag: expected,
                    content: schema,
                },
                Value::Tag(actual, value),
            ) => expected == actual && schema.validate(value),
            (Schema::Array { content: schema }, Value::Array(items)) => {
                items.iter().all(|item| schema.validate(item))
            }
            (Schema::Map { content: schema }, Value::Map(values)) => {
                let schemas: HashMap<_, _> = schema.iter().cloned().collect();
                let values: HashMap<_, _> = values.iter().cloned().collect();
                let all_keys_valid = values.iter().all(|(key, value)| {
                    let schema = schemas.get(key);
                    schema.map_or(false, |schema| schema.validate(value))
                });
                let all_required_present = schemas.iter().all(|(key, schema)| {
                    values.contains_key(key) || matches!(**schema, Schema::Optional { .. })
                });
                all_keys_valid && all_required_present
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_array_integers() {
        let schema = Schema::Array {
            content: Box::new(Schema::Integer),
        };
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
        let schema = Schema::Array {
            content: Box::new(Schema::Integer),
        };
        let value = Value::Text("hello world".to_string());
        let valid = schema.validate(&value);
        assert!(!valid);
    }

    #[test]
    fn test_schema_map() {
        let schema = Schema::Map {
            content: vec![
                ("one".into(), Box::new(Schema::Integer)),
                ("two".into(), Box::new(Schema::Integer)),
                ("three".into(), Box::new(Schema::Float)),
            ],
        };
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
        let schema = Schema::Map {
            content: vec![
                ("one".into(), Box::new(Schema::Integer)),
                ("two".into(), Box::new(Schema::Integer)),
                ("three".into(), Box::new(Schema::Float)),
                (
                    "four".to_string(),
                    Box::new(Schema::Optional {
                        content: Box::new(Schema::Text),
                    }),
                ),
            ],
        };
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
        let schema = Schema::Map {
            content: vec![
                ("one".into(), Box::new(Schema::Integer)),
                ("two".into(), Box::new(Schema::Integer)),
            ],
        };
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
        let schema = Schema::Optional {
            content: Box::new(Schema::Text),
        };
        let value = Value::Null;
        let valid = schema.validate(&value);
        assert!(valid);
    }

    #[test]
    fn test_schema_optional_text() {
        let schema = Schema::Optional {
            content: Box::new(Schema::Text),
        };
        let value = Value::Text("hello world".to_string());
        let valid = schema.validate(&value);
        assert!(valid);
    }

    #[test]
    fn test_schema_any() {
        let schema = Schema::Any;
        let value = Value::Text("hello world".to_string());
        let valid = schema.validate(&value);
        assert!(valid);
    }
}
