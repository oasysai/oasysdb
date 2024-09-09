use super::*;
use crate::protos;

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
