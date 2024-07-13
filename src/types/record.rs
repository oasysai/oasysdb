use crate::types::err::{Error, ErrorCode};
use half::f16;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyRow;
use sqlx::database::HasValueRef;
use sqlx::{Database, Decode, Row, Type};
use std::collections::HashMap;
use std::error::Error as StandardError;

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
    Integer(usize),
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
        if let Ok(integer) = value.parse::<usize>() {
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

impl From<usize> for RecordData {
    fn from(value: usize) -> Self {
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

impl<'r, DB: Database> Decode<'r, DB> for Vector
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as HasValueRef<'r>>::ValueRef,
    ) -> Result<Vector, Box<dyn StandardError + Send + Sync + 'static>> {
        let value = <&str as Decode<DB>>::decode(value)?;
        let vector: Vec<f32> = serde_json::from_str(value)?;
        Ok(Vector(vector.into_iter().map(f16::from_f32).collect()))
    }
}

impl<DB> Type<DB> for Vector
where
    DB: Database,
    &'static str: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <&str as Type<DB>>::type_info()
    }
}

impl RowOps for Vector {
    fn from_row(
        column_name: impl Into<String>,
        row: &AnyRow,
    ) -> Result<Self, Error> {
        let column: String = column_name.into();
        let vector = row.try_get::<Self, &str>(&column).map_err(|_| {
            let code = ErrorCode::InvalidVector;
            let message = "Unable to get vector from the row.";
            Error::new(code, message)
        })?;

        Ok(vector)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for RecordData
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as HasValueRef<'r>>::ValueRef,
    ) -> Result<RecordData, Box<dyn StandardError + Send + Sync + 'static>>
    {
        let value = <&str as Decode<DB>>::decode(value)?;
        Ok(RecordData::from(value))
    }
}

impl<DB> Type<DB> for RecordData
where
    DB: Database,
    &'static str: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <&str as Type<DB>>::type_info()
    }
}

impl RowOps for Option<RecordData> {
    fn from_row(
        column_name: impl Into<String>,
        row: &AnyRow,
    ) -> Result<Self, Error> {
        let column: String = column_name.into();
        Ok(row.try_get::<Self, &str>(&column).unwrap_or_default())
    }
}
