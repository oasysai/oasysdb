use super::*;
use uuid::Uuid;

/// Record identifier.
///
/// OasysDB should be able to deal with a lot of writes and deletes. Using UUID
/// version 4 to allow us to generate a lot of IDs with very low probability
/// of collision.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RecordID(Uuid);

/// Metadata value.
///
/// OasysDB doesn't support nested objects in metadata for performance reasons.
/// We only need to support primitive types for metadata.
#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Text(String),
    Number(f64),
    Boolean(bool),
}

/// OasysDB vector record.
///
/// This is the main data structure for OasysDB. It contains the vector data
/// and metadata of the record. Metadata is a key-value store that can be used
/// to store additional information about the vector.
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    vector: Vector,
    metadata: HashMap<String, Value>,
}
