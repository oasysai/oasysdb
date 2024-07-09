use half::f16;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Column name of the SQL data source table.
pub type ColumnName = String;

/// ID type for records in the index from the data source.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize, Deserialize)]
#[derive(PartialEq, Eq, Hash)]
pub enum RecordID {
    /// Auto-incrementing integer ID (Most efficient).
    Integer(usize),
    /// String as ID (Not efficient).
    String(String),
    /// Universally Unique ID (Less efficient).
    UUID(Uuid),
}

/// Record type stored in the index based on the
/// configuration and data source.
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    vector: Vector,
    data: Option<HashMap<ColumnName, RecordData>>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Vector data type stored in the index.
pub struct Vector(Vec<f16>);

/// Data types supported as metadata in the index.
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RecordData {
    Boolean(bool),
    Float(f32),
    Integer(usize),
    String(String),
}
