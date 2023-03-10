//!Fluentd record definitions.

use core::fmt::Debug;
use std::collections::HashMap;

use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

#[derive(Clone, PartialEq)]
/// HashMap object for fluent record.
pub struct Map(HashMap<String, Value>);

impl Map {
    /// Create an empty Map object.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Create an Map object with existed key-values
    pub fn new_with(map: HashMap<String, Value>) -> Self {
        let mut m = HashMap::new();
        for (k, v) in map.into_iter() {
            m.insert(k, v);
        }
        Self(m)
    }
}

#[macro_export]
/// Create a Map object from a list of key-value pairs.
///
/// ## Example
///
/// ```
/// use std::collections::HashMap;
///
/// use tokio_fluent::record_map;
/// use tokio_fluent::record::{Map, Value};
///
/// let map = record_map!(
///     "name".to_string() => "John".into(),
///     "age".to_string() => 22.into(),
///     "scores".to_string() => [70, 80].into_iter().map(|e| e.into()).collect::<Vec<_>>().into(),
/// );
/// assert_eq!(map["name"], Value::from("John"));
/// assert_eq!(map["age"], Value::from(22));
/// assert_eq!(map["scores"], Value::from([70, 80].into_iter().map(|e| e.into()).collect::<Vec<_>>()));
/// ```
macro_rules! record_map {
    ($($key:expr => $field:expr,)+) => { record_map!($($key => $field),+) };
    ($($key:expr => $field:expr),*) => {
        {
            let mut map: HashMap<String, Value> = HashMap::new();
            $(
                map.insert($key, $field);
            )+
            Map::new_with(map)
        }
    };
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

#[derive(Clone, PartialEq)]
/// Value object for HashMap of a fluentd record.
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
        Self::Int(value as _)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Self::Int(value as _)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Uint(value as _)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Uint(value)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Uint(value as _)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_map() {
        let got = record_map!(
            "name".to_string() => "John".into(),
            "age".to_string() => 22.into(),
            "scores".to_string() => [70, 80].into_iter().map(|e| e.into()).collect::<Vec<_>>().into(),
        );

        let mut want = Map::new();
        want.insert("name".to_string(), "John".into());
        want.insert("age".to_string(), 22.into());
        want.insert(
            "scores".to_string(),
            [70, 80]
                .into_iter()
                .map(|e| e.into())
                .collect::<Vec<_>>()
                .into(),
        );
        assert_eq!(got, want);
    }
}
