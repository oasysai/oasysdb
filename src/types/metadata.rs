use super::*;
use crate::protos;
use std::collections::HashMap;

/// Metadata of a vector record.
///
/// Metadata is a key-value store that can be used to store additional context
/// about a vector such as the source document of the vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata(HashMap<String, Option<Value>>);

impl From<HashMap<String, protos::Value>> for Metadata {
    fn from(metadata: HashMap<String, protos::Value>) -> Self {
        let mut map = HashMap::new();
        for (key, value) in metadata {
            map.insert(key, value.into());
        }

        Self(map)
    }
}

/// Metadata value of a vector record.
///
/// Metadata value only supports primitive data types like string, integer,
/// float, and boolean. We do this to ensure that the metadata stays small
/// and easy to manage.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Text(String),
    Integer(i32),
    Float(f32),
    Boolean(bool),
}

impl From<protos::Value> for Option<Value> {
    fn from(value: protos::Value) -> Self {
        type Data = protos::value::Data;
        match value.data {
            Some(Data::Text(s)) => Some(Value::Text(s)),
            Some(Data::Integer(i)) => Some(Value::Integer(i)),
            Some(Data::Float(f)) => Some(Value::Float(f)),
            Some(Data::Boolean(b)) => Some(Value::Boolean(b)),
            None => None,
        }
    }
}
