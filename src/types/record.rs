use super::*;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Record identifier.
///
/// OasysDB should be able to deal with a lot of writes and deletes. Using UUID
/// version 4 to allow us to generate a lot of IDs with very low probability
/// of collision.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RecordID(Uuid);

impl RecordID {
    /// Generate a new random record ID using UUID v4.
    pub fn new() -> Self {
        RecordID(Uuid::new_v4())
    }
}

impl fmt::Display for RecordID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RecordID {
    type Err = Status;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RecordID(Uuid::try_parse(s).map_err(|_| {
            let message = "Record ID should be a string-encoded UUID";
            Status::invalid_argument(message)
        })?))
    }
}

/// Metadata value.
///
/// OasysDB doesn't support nested objects in metadata for performance reasons.
/// We only need to support primitive types for metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Value {
    Text(String),
    Number(f64),
    Boolean(bool),
}

impl From<Value> for protos::Value {
    fn from(value: Value) -> Self {
        type ProtoValue = protos::value::Value;
        let value = match value {
            Value::Text(text) => ProtoValue::Text(text),
            Value::Number(number) => ProtoValue::Number(number),
            Value::Boolean(boolean) => ProtoValue::Boolean(boolean),
        };

        protos::Value { value: Some(value) }
    }
}

impl TryFrom<protos::Value> for Value {
    type Error = Status;
    fn try_from(value: protos::Value) -> Result<Self, Self::Error> {
        type ProtoValue = protos::value::Value;
        match value.value {
            Some(ProtoValue::Text(text)) => Ok(Value::Text(text)),
            Some(ProtoValue::Number(number)) => Ok(Value::Number(number)),
            Some(ProtoValue::Boolean(boolean)) => Ok(Value::Boolean(boolean)),
            None => Err(Status::invalid_argument("Metadata value is required")),
        }
    }
}

/// OasysDB vector record.
///
/// This is the main data structure for OasysDB. It contains the vector data
/// and metadata of the record. Metadata is a key-value store that can be used
/// to store additional information about the vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub vector: Vector,
    pub metadata: HashMap<String, Value>,
}

impl From<Record> for protos::Record {
    fn from(value: Record) -> Self {
        let vector = value.vector.into();
        let metadata = value
            .metadata
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect();

        protos::Record { vector: Some(vector), metadata }
    }
}

impl TryFrom<protos::Record> for Record {
    type Error = Status;
    fn try_from(value: protos::Record) -> Result<Self, Self::Error> {
        let vector = match value.vector {
            Some(vector) => Vector::try_from(vector)?,
            None => {
                let message = "Vector data should not be empty";
                return Err(Status::invalid_argument(message));
            }
        };

        let metadata = value
            .metadata
            .into_iter()
            .map(|(k, v)| Ok((k, v.try_into()?)))
            .collect::<Result<HashMap<String, Value>, Self::Error>>()?;

        Ok(Record { vector, metadata })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random;

    impl Value {
        pub fn random() -> Self {
            Value::Number(random::<f64>())
        }
    }

    impl Record {
        pub fn random(dimension: usize) -> Self {
            let mut metadata = HashMap::new();
            metadata.insert("key".to_string(), Value::random());
            Record { vector: Vector::random(dimension), metadata }
        }
    }
}
