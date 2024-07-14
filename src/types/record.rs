use crate::types::err::{Error, ErrorCode};
use half::f16;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyRow;
use sqlx::postgres::any::AnyTypeInfoKind as SQLType;
use sqlx::{Row, ValueRef};
use std::collections::HashMap;

/// Column name of the SQL data source table.
pub type ColumnName = String;

/// ID type for records in the index from the data source.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct RecordID(pub u32);

/// Record type stored in the index based on the
/// configuration and data source.
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    /// Vector embedding.
    pub vector: Vector,
    /// Additional metadata of the record.
    pub data: HashMap<ColumnName, Option<RecordData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Vector data type stored in the index.
pub struct Vector(pub Vec<f16>);

impl Vector {
    /// Returns the vector data as a vector of f32.
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.clone().into_iter().map(f16::to_f32).collect()
    }

    /// Returns the dimension of the vector.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<f32>> for Vector {
    fn from(value: Vec<f32>) -> Self {
        Vector(value.into_iter().map(f16::from_f32).collect())
    }
}

/// Data types supported as metadata in the index.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordData {
    Boolean(bool),
    Float(f32),
    Integer(isize),
    String(String),
}

// RecordData interoperability with primitive types.

impl From<String> for RecordData {
    fn from(value: String) -> Self {
        RecordData::from(value.as_str())
    }
}

impl From<&str> for RecordData {
    fn from(value: &str) -> Self {
        // Parsing integer must be done before float.
        // Since integer can be parsed as float but not vice versa.
        if let Ok(integer) = value.parse::<isize>() {
            return integer.into();
        }

        if let Ok(float) = value.parse::<f32>() {
            return float.into();
        }

        if let Ok(boolean) = value.parse::<bool>() {
            return boolean.into();
        }

        RecordData::String(value.to_string())
    }
}

impl From<f32> for RecordData {
    fn from(value: f32) -> Self {
        RecordData::Float(value)
    }
}

impl From<isize> for RecordData {
    fn from(value: isize) -> Self {
        RecordData::Integer(value)
    }
}

impl From<bool> for RecordData {
    fn from(value: bool) -> Self {
        RecordData::Boolean(value)
    }
}

pub(crate) trait RowOps {
    /// Retrieves data from the row based on the column name.
    fn from_row(
        column_name: impl Into<String>,
        row: &AnyRow,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

impl RowOps for RecordID {
    fn from_row(
        column_name: impl Into<String>,
        row: &AnyRow,
    ) -> Result<Self, Error> {
        let column_name: String = column_name.into();
        let id = row.try_get::<i32, &str>(&column_name).map_err(|_| {
            let code = ErrorCode::InvalidID;
            let message = "Unable to get integer ID from the row.";
            Error::new(code, message)
        })?;

        Ok(RecordID(id as u32))
    }
}

impl RowOps for Vector {
    fn from_row(
        column_name: impl Into<String>,
        row: &AnyRow,
    ) -> Result<Self, Error> {
        let column: String = column_name.into();
        let value = row.try_get_raw::<&str>(&column)?;
        let value_type = value.type_info().kind();

        if value_type == SQLType::Null {
            let code = ErrorCode::InvalidVector;
            let message = "Vector must not be empty or null.";
            return Err(Error::new(code, message));
        }

        match value_type {
            SQLType::Text => {
                let value = row.try_get::<String, &str>(&column)?;
                let vector: Vec<f32> = serde_json::from_str(&value)?;
                Ok(Vector::from(vector))
            }
            SQLType::Blob => {
                let value = row.try_get::<Vec<u8>, &str>(&column)?;
                let vector: Vec<f32> = bincode::deserialize(&value)?;
                Ok(Vector::from(vector))
            }
            _ => {
                let code = ErrorCode::InvalidVector;
                let message = "Vector must be stored as JSON string or blob.";
                Err(Error::new(code, message))
            }
        }
    }
}

impl RowOps for Option<RecordData> {
    fn from_row(
        column_name: impl Into<String>,
        row: &AnyRow,
    ) -> Result<Self, Error> {
        let column: String = column_name.into();
        let value = row.try_get_raw::<&str>(&column)?;
        let value_type = value.type_info().kind();

        if value_type == SQLType::Null {
            return Ok(None);
        }

        if value_type.is_integer() {
            let value: i64 = row.try_get::<i64, &str>(&column)?;
            return Ok(Some(RecordData::Integer(value as isize)));
        }

        // Handle types other than null and integer below.

        let data = match value_type {
            SQLType::Text => {
                let value = row.try_get::<String, &str>(&column)?;
                RecordData::String(value.to_string())
            }
            SQLType::Bool => {
                let value: bool = row.try_get::<bool, &str>(&column)?;
                RecordData::Boolean(value)
            }
            SQLType::Real => {
                let value: f32 = row.try_get::<f32, &str>(&column)?;
                RecordData::Float(value)
            }
            SQLType::Double => {
                let value: f64 = row.try_get::<f64, &str>(&column)?;
                RecordData::Float(value as f32)
            }
            _ => {
                let code = ErrorCode::InvalidMetadata;
                let message = "Unsupported type for OasysDB metadata.";
                return Err(Error::new(code, message));
            }
        };

        Ok(Some(data))
    }
}
