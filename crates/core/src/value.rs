use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
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

impl Serialize for Value {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Integer(i) => serializer.serialize_i128(*i),
            Value::Bytes(b) => serializer.serialize_bytes(b),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::Text(s) => serializer.serialize_str(s),
            Value::Boolean(b) => serializer.serialize_bool(*b),
            Value::Null => serializer.serialize_unit(),
            Value::Tag(tag, val) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(&tag.to_string(), val)?;
                map.end()
            }
            Value::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Map(map) => {
                let mut ser_map = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    ser_map.serialize_entry(k, v)?;
                }
                ser_map.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter,
            ) -> fmt::Result {
                formatter.write_str("a valid Value representation")
            }

            fn visit_i64<E>(
                self,
                v: i64,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Integer(v as i128))
            }

            fn visit_u64<E>(
                self,
                v: u64,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Integer(v as i128))
            }

            fn visit_f64<E>(
                self,
                v: f64,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_str<E>(
                self,
                v: &str,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Text(v.to_string()))
            }

            fn visit_bool<E>(
                self,
                v: bool,
            ) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Boolean(v))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_seq<A>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut elements = Vec::new();
                while let Some(element) = seq.next_element()? {
                    elements.push(element);
                }
                Ok(Value::Array(elements))
            }

            fn visit_map<A>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some((key, value)) = map.next_entry()? {
                    values.push((key, value));
                }
                Ok(Value::Map(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
