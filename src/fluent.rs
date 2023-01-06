use core::fmt::Debug;
use std::collections::HashMap;

use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};

#[derive(Clone)]
/// HashMap object for fluent record.
pub struct Map(HashMap<String, Value>);

impl Map {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn new_with_values(map: HashMap<String, Value>) -> Self {
        let mut m = HashMap::new();
        for (k, v) in map.into_iter() {
            m.insert(k, v);
        }
        Self(m)
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl core::ops::Deref for Map {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub enum Value {
    /// Boolean
    Bool(bool),
    /// Integer
    Int(i64),
    /// Unsigned integer
    Uint(u64),
    /// Float
    Float(f64),
    /// String
    Str(String),
    /// Object
    Object(Map),
    /// Array
    Array(Vec<Value>),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int(value.into())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Uint(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float(value.into())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl From<Map> for Value {
    fn from(value: Map) -> Self {
        Self::Object(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Self::Array(value)
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Bool(value) => f.write_fmt(format_args!("{}", value)),
            Value::Int(value) => f.write_fmt(format_args!("{}", value)),
            Value::Uint(value) => f.write_fmt(format_args!("{}", value)),
            Value::Float(value) => f.write_fmt(format_args!("{}", value)),
            Value::Str(value) => f.write_fmt(format_args!("{}", value)),
            Value::Object(value) => f.write_fmt(format_args!("{:?}", value)),
            Value::Array(value) => f.write_fmt(format_args!("{:?}", value)),
        }
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Bool(value) => serializer.serialize_bool(*value),
            Value::Int(value) => serializer.serialize_i64(*value),
            Value::Uint(value) => serializer.serialize_u64(*value),
            Value::Float(value) => serializer.serialize_f64(*value),
            Value::Str(value) => serializer.serialize_str(value),
            Value::Object(value) => {
                let mut map = serializer.serialize_map(Some(value.len()))?;
                for (k, v) in value.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Value::Array(value) => {
                let mut seq = serializer.serialize_seq(Some(value.len()))?;
                for e in value {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
        }
    }
}
