use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;
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

impl<'de> Deserializer<'de> for Value {
    type Error = de::value::Error;

    fn deserialize_any<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Integer(v) => visitor.visit_i128(v),
            Value::Bytes(v) => visitor.visit_byte_buf(v),
            Value::Float(v) => visitor.visit_f64(v),
            Value::Text(v) => visitor.visit_string(v),
            Value::Boolean(v) => visitor.visit_bool(v),
            Value::Null => visitor.visit_unit(),
            Value::Tag(tag, boxed) => visitor.visit_map(TagDeserializer { tag, value: *boxed }),
            Value::Array(v) => visitor.visit_seq(SeqDeserializer::new(v)),
            Value::Map(v) => visitor.visit_map(MapDeserializer::new(v)),
        }
    }

    fn deserialize_u8<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i8<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u16<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i16<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i32<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_u64<V: de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self {
            Value::Integer(n) if n >= 0 && n <= u64::MAX as i128 => visitor.visit_u64(n as u64),
            _ => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other("expected a u64-compatible integer"),
                &"u64",
            )),
        }
    }

    fn deserialize_i64<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Integer(n) if n >= i64::MIN as i128 && n <= i64::MAX as i128 => {
                visitor.visit_i64(n as i64)
            }
            _ => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other("expected an i64-compatible integer"),
                &"i64",
            )),
        }
    }

    fn deserialize_option<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            other => visitor.visit_some(other),
        }
    }

    fn deserialize_seq<V>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Bytes(v) => visitor.visit_seq(SeqDeserializer::new(
                v.into_iter().map(|b| Value::Integer(b as i128)).collect(),
            )),
            Value::Array(v) => visitor.visit_seq(SeqDeserializer::new(v)),
            _ => Err(serde::de::Error::invalid_type(
                serde::de::Unexpected::Other("not a sequence"),
                &"a sequence",
            )),
        }
    }

    forward_to_deserialize_any! {
        bool i128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct
        tuple tuple_struct map struct enum identifier ignored_any
    }
}

struct SeqDeserializer {
    values: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(values: Vec<Value>) -> Self {
        Self {
            values: values.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = de::value::Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

struct MapDeserializer {
    entries: std::vec::IntoIter<(String, Box<Value>)>,
    current: Option<(String, Box<Value>)>,
}

impl MapDeserializer {
    fn new(entries: Vec<(String, Box<Value>)>) -> Self {
        Self {
            entries: entries.into_iter(),
            current: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = de::value::Error;

    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.entries.next() {
            self.current = Some((key.clone(), value));
            seed.deserialize(Value::Text(key)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some((_, value)) = self.current.take() {
            seed.deserialize(*value)
        } else {
            Err(de::Error::custom("value is missing"))
        }
    }
}

struct TagDeserializer {
    tag: u64,
    value: Value,
}

impl<'de> MapAccess<'de> for TagDeserializer {
    type Error = de::value::Error;

    fn next_key_seed<K>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        seed.deserialize(Value::Text("tag".into())).map(Some)
    }

    fn next_value_seed<V>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(Value::Integer(self.tag as i128))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        string: String,
        bytes: Vec<u8>,
        optional_null: Option<String>,
        optional_value: Option<String>,
        bool_false: bool,
        bool_true: bool,
        u8_min: u8,
        u8_max: u8,
        u16_min: u16,
        u16_max: u16,
        u32_min: u32,
        u32_max: u32,
        u64_min: u64,
        u64_max: u64,
        i8_min: i8,
        i8_max: i8,
        i16_min: i16,
        i16_max: i16,
        i32_min: i32,
        i32_max: i32,
        i64_min: i64,
        i64_max: i64,
        f32_min: f32,
        f32_max: f32,
        f64_min: f64,
        f64_max: f64,
        vec: Vec<u64>,
        deep: TestStructDeep,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStructDeep {
        a: String,
        b: bool,
        c: u8,
    }

    #[test]
    #[should_panic]
    fn test_deserialize_fail() {
        TestStruct::deserialize(Value::Null).unwrap();
    }

    #[test]
    fn test_deserialize() {
        let actual = TestStruct::deserialize(Value::Map(vec![
            ("string".into(), Value::Text("hello".into()).into()),
            ("bytes".into(), Value::Bytes(vec![1, 2, 3, 4, 5]).into()),
            ("optional_null".into(), Value::Null.into()),
            ("optional_value".into(), Value::Text("hola".into()).into()),
            ("bool_false".into(), Value::Boolean(false).into()),
            ("bool_true".into(), Value::Boolean(true).into()),
            ("u8_min".into(), Value::Integer(u8::MIN as i128).into()),
            ("u8_max".into(), Value::Integer(u8::MAX as i128).into()),
            ("u16_min".into(), Value::Integer(u16::MIN as i128).into()),
            ("u16_max".into(), Value::Integer(u16::MAX as i128).into()),
            ("u32_min".into(), Value::Integer(u32::MIN as i128).into()),
            ("u32_max".into(), Value::Integer(u32::MAX as i128).into()),
            ("u64_min".into(), Value::Integer(u64::MIN as i128).into()),
            ("u64_max".into(), Value::Integer(u64::MAX as i128).into()),
            ("i8_min".into(), Value::Integer(i8::MIN as i128).into()),
            ("i8_max".into(), Value::Integer(i8::MAX as i128).into()),
            ("i16_min".into(), Value::Integer(i16::MIN as i128).into()),
            ("i16_max".into(), Value::Integer(i16::MAX as i128).into()),
            ("i32_min".into(), Value::Integer(i32::MIN as i128).into()),
            ("i32_max".into(), Value::Integer(i32::MAX as i128).into()),
            ("i64_min".into(), Value::Integer(i64::MIN as i128).into()),
            ("i64_max".into(), Value::Integer(i64::MAX as i128).into()),
            ("f32_min".into(), Value::Float(f32::MIN.into()).into()),
            ("f32_max".into(), Value::Float(f32::MAX.into()).into()),
            ("f64_min".into(), Value::Float(f64::MIN).into()),
            ("f64_max".into(), Value::Float(f64::MAX).into()),
            (
                "vec".into(),
                Value::Array(vec![
                    Value::Integer(u8::MAX as i128),
                    Value::Integer(u16::MAX as i128),
                    Value::Integer(u32::MAX as i128),
                    Value::Integer(u64::MAX as i128),
                ])
                .into(),
            ),
            (
                "deep".into(),
                Value::Map(vec![
                    ("a".into(), Value::Text("world".into()).into()),
                    ("b".into(), Value::Boolean(true).into()),
                    ("c".into(), Value::Integer(u8::MAX as i128).into()),
                ])
                .into(),
            ),
        ]))
        .unwrap();
        let expected = TestStruct {
            string: "hello".into(),
            bytes: vec![1, 2, 3, 4, 5],
            optional_null: None,
            optional_value: Some("hola".into()),
            bool_false: false,
            bool_true: true,
            u8_min: u8::MIN,
            u8_max: u8::MAX,
            u16_min: u16::MIN,
            u16_max: u16::MAX,
            u32_min: u32::MIN,
            u32_max: u32::MAX,
            u64_min: u64::MIN,
            u64_max: u64::MAX,
            i8_min: i8::MIN,
            i8_max: i8::MAX,
            i16_min: i16::MIN,
            i16_max: i16::MAX,
            i32_min: i32::MIN,
            i32_max: i32::MAX,
            i64_min: i64::MIN,
            i64_max: i64::MAX,
            f32_min: f32::MIN,
            f32_max: f32::MAX,
            f64_min: f64::MIN,
            f64_max: f64::MAX,
            vec: vec![u8::MAX as u64, u16::MAX as u64, u32::MAX as u64, u64::MAX],
            deep: TestStructDeep {
                a: "world".into(),
                b: true,
                c: u8::MAX,
            },
        };
        assert_eq!(actual, expected);
    }
}
