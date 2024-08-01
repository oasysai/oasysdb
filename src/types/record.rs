use crate::types::err::{Error, ErrorCode};
use half::f16;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyRow;
use sqlx::postgres::any::AnyTypeInfoKind as SQLType;
use sqlx::{Row, ValueRef};
use std::collections::HashMap;

/// Column name in the SQL data source table.
pub type ColumnName = String;

/// Integer-based ID for each record in the index.
///
/// For this to work properly with SQL as the data source, the column
/// containing the primary key must be a unique auto-incrementing integer.
/// Auto-incrementing integer type is important to allow the index to be
/// updated incrementally.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct RecordID(pub u32);

/// Record data type stored in non-PQ indices.
///
/// This data type contains the vector embedding and additional metadata
/// which depends on the source configuration. This data type is compatible
/// with the standard SQL row type.
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    /// Vector embedding.
    pub vector: Vector,
    /// Additional metadata of the record.
    pub data: HashMap<ColumnName, Option<DataValue>>,
}

/// Record data type stored in PQ-based indices.
///
/// This data type is very similar to the standard Record type
/// except that the vector stored within is quantized using the
/// Product Quantization (PQ) method.
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordPQ {
    /// Product quantized embedding.
    pub vector: VectorPQ,
    /// Additional metadata of the record.
    pub data: HashMap<ColumnName, Option<DataValue>>,
}

/// Vector data type for non-PQ indices.
///
/// This data type uses the half-precision floating-point format
/// to store the vector data to reduce the memory footprint by half
/// compared to the standard f32 format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(PartialEq, PartialOrd)]
pub struct Vector(pub Box<[f16]>);

impl Vector {
    /// Returns the vector data as a vector of f32.
    pub fn to_vec(&self) -> Vec<f32> {
        // Don't use parallel iterator here since it actually
        // slows it down significantly.
        self.0.iter().map(|v| v.to_f32()).collect()
    }

    /// Returns the length of the vector.
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
        Vector(value.into_par_iter().map(f16::from_f32).collect())
    }
}

/// Product quantized vector data type stored in the index.
///
/// PQ is a method used to reduce the memory footprint of the vector
/// data by dividing the vector into sub-vectors and quantizing them.
/// When performing similarity search, the quantized vectors are used
/// to reconstruct the original vector along with the codebook.
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorPQ(pub Box<[u8]>);

impl VectorPQ {
    /// Returns the vector data as a vector of u8.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl From<Vec<u8>> for VectorPQ {
    fn from(value: Vec<u8>) -> Self {
        VectorPQ(value.into_boxed_slice())
    }
}

/// Data types supported as metadata in the index.
///
/// These are the corresponding SQL data types of the metadata:
/// - Boolean: BOOL
/// - Float: REAL | DOUBLE (Both converted to F32)
/// - Integer: SMALLINT | INT | BIGINT
/// - String: TEXT
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataValue {
    Boolean(bool),
    Float(f32),
    Integer(isize),
    String(String),
}

// DataValue interoperability with primitive types.

impl From<String> for DataValue {
    fn from(value: String) -> Self {
        DataValue::from(value.as_str())
    }
}

impl From<&str> for DataValue {
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

        let match_quotes = |c: char| c == '\"' || c == '\'';
        let value = value
            .trim_start_matches(match_quotes)
            .trim_end_matches(match_quotes)
            .to_string();

        DataValue::String(value)
    }
}

impl From<f32> for DataValue {
    fn from(value: f32) -> Self {
        DataValue::Float(value)
    }
}

impl From<isize> for DataValue {
    fn from(value: isize) -> Self {
        DataValue::Integer(value)
    }
}

impl From<bool> for DataValue {
    fn from(value: bool) -> Self {
        DataValue::Boolean(value)
    }
}

pub(crate) trait RowOps {
    /// Retrieves data from the row based on the column name.
    /// - `column_name`: Name of the column to retrieve data from.
    /// - `row`: SQL row containing the data.
    fn from_row(
        column_name: impl Into<ColumnName>,
        row: &AnyRow,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

impl RowOps for RecordID {
    fn from_row(
        column_name: impl Into<ColumnName>,
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
        column_name: impl Into<ColumnName>,
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

impl RowOps for Option<DataValue> {
    fn from_row(
        column_name: impl Into<ColumnName>,
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
            return Ok(Some(DataValue::Integer(value as isize)));
        }

        // Handle types other than null and integer below.

        let data = match value_type {
            SQLType::Text => {
                let value = row.try_get::<String, &str>(&column)?;
                DataValue::String(value.to_string())
            }
            SQLType::Bool => {
                let value: bool = row.try_get::<bool, &str>(&column)?;
                DataValue::Boolean(value)
            }
            SQLType::Real => {
                let value: f32 = row.try_get::<f32, &str>(&column)?;
                DataValue::Float(value)
            }
            SQLType::Double => {
                let value: f64 = row.try_get::<f64, &str>(&column)?;
                DataValue::Float(value as f32)
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
