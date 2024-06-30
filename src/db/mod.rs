use crate::proto;
use crate::types::*;
use array::{BooleanArray, Float32Array, Int32Array, ListArray, StringArray};
use arrow::array::{self, Array};
use arrow::datatypes::DataType;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use arrow_schema::{Field, Fields, Schema};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, RwLock as Lock};
use tonic::{Request, Response, Status};

mod collection;
mod collection_utils;
mod database;
mod database_service;

pub use collection::*;
pub use database::*;

type ProtoValue = proto::data::Value;

/// A trait for objects that own a state that should be persisted to disk.
/// - `T`: Type of the state object.
///
/// Please refer to the implementation of the StateMachine trait for
/// Database and Collection for more details.
pub trait StateMachine<T> {
    /// Initializes the state object and persists it to a file.
    /// This method should be called only once when the object is created.
    fn initialize_state(path: impl Into<PathBuf>) -> Result<T, Error>;

    /// Reads the state object from a file.
    fn read_state(path: impl Into<PathBuf>) -> Result<T, Error>;

    /// Returns a reference to the state object.
    fn state(&self) -> Result<T, Error>;

    /// Persists the state object to a file.
    fn persist_state(&self) -> Result<(), Error>;
}

pub trait ArrayUtils {
    fn from_values(
        values: Vec<Option<ProtoValue>>,
    ) -> Result<Arc<dyn Array>, Error>;

    /// Creates an array filled with null values.
    fn null_array(len: usize) -> Arc<dyn Array>;
}

pub trait ListArrayUtils {
    fn from_vectors(values: Vec<Vec<f32>>) -> Arc<dyn Array>;
}

impl ArrayUtils for BooleanArray {
    fn from_values(
        values: Vec<Option<ProtoValue>>,
    ) -> Result<Arc<dyn Array>, Error> {
        let parse_boolean = |value: Option<ProtoValue>| match value {
            Some(ProtoValue::BooleanValue(value)) => Some(value),
            _ => None,
        };

        let values: Vec<Option<bool>> =
            values.into_par_iter().map(parse_boolean).collect();
        Ok(Arc::new(BooleanArray::from(values)))
    }

    fn null_array(len: usize) -> Arc<dyn Array> {
        Arc::new(BooleanArray::from(vec![None; len]))
    }
}

impl ArrayUtils for Float32Array {
    fn from_values(
        values: Vec<Option<ProtoValue>>,
    ) -> Result<Arc<dyn Array>, Error> {
        let parse_float = |value: Option<ProtoValue>| match value {
            Some(ProtoValue::FloatValue(value)) => Some(value),
            _ => None,
        };

        let values: Vec<Option<f32>> =
            values.into_par_iter().map(parse_float).collect();
        Ok(Arc::new(Float32Array::from(values)))
    }

    fn null_array(len: usize) -> Arc<dyn Array> {
        Arc::new(Float32Array::from(vec![None; len]))
    }
}

impl ArrayUtils for Int32Array {
    fn from_values(
        values: Vec<Option<ProtoValue>>,
    ) -> Result<Arc<dyn Array>, Error> {
        let parse_int = |value: Option<ProtoValue>| match value {
            Some(ProtoValue::IntegerValue(value)) => Some(value),
            _ => None,
        };

        let values: Vec<Option<i32>> =
            values.into_par_iter().map(parse_int).collect();
        Ok(Arc::new(Int32Array::from(values)))
    }

    fn null_array(len: usize) -> Arc<dyn Array> {
        Arc::new(Int32Array::from(vec![None; len]))
    }
}

impl ArrayUtils for StringArray {
    fn from_values(
        values: Vec<Option<ProtoValue>>,
    ) -> Result<Arc<dyn Array>, Error> {
        let parse_string = |value: Option<ProtoValue>| match value {
            Some(ProtoValue::StringValue(value)) => Some(value),
            _ => None,
        };

        let values: Vec<Option<String>> =
            values.into_par_iter().map(parse_string).collect();
        Ok(Arc::new(StringArray::from(values)))
    }

    fn null_array(len: usize) -> Arc<dyn Array> {
        let source: Vec<Option<String>> = vec![None; len];
        Arc::new(StringArray::from(source))
    }
}

impl ArrayUtils for ListArray {
    fn from_values(
        values: Vec<Option<ProtoValue>>,
    ) -> Result<Arc<dyn Array>, Error> {
        let parse_vector = |value: Option<ProtoValue>| match value {
            Some(ProtoValue::VectorValue(value)) => Some(value.values),
            _ => None,
        };

        let values: Vec<Option<Vec<f32>>> =
            values.into_par_iter().map(parse_vector).collect();

        // Find the dimension of the vector.
        let dimension = values
            .clone()
            .into_par_iter()
            .map(|value| value.unwrap_or_default().len())
            .max()
            // 1024 is the default capacity for generic array builders.
            .unwrap_or(1024);

        // Create builders to construct the ListArray.
        let mut list_builder = {
            let float_builder = Float32Array::builder(dimension);
            let field = Field::new("element", DataType::Float32, false);
            array::ListBuilder::new(float_builder).with_field(field)
        };

        // Insert values into the builder.
        for value in values {
            match value {
                Some(values) => {
                    list_builder.values().append_slice(&values);
                    list_builder.append(true);
                }
                None => list_builder.append(false),
            }
        }

        let array = list_builder.finish();
        Ok(Arc::new(array))
    }

    fn null_array(len: usize) -> Arc<dyn Array> {
        let mut builder = {
            // We can use 0 capacity since we are not going to append any values.
            let float_builder = Float32Array::builder(0);
            let field = Field::new("element", DataType::Float32, false);
            array::ListBuilder::new(float_builder).with_field(field)
        };

        for _ in 0..len {
            builder.append(false);
        }

        let array = builder.finish();
        Arc::new(array)
    }
}

impl ListArrayUtils for ListArray {
    fn from_vectors(values: Vec<Vec<f32>>) -> Arc<dyn Array> {
        let dimension = values[0].len();

        let mut list_builder = {
            let float_builder = Float32Array::builder(dimension);
            let field = Field::new("element", DataType::Float32, false);
            array::ListBuilder::new(float_builder).with_field(field)
        };

        for value in values {
            list_builder.values().append_slice(&value);
            list_builder.append(true);
        }

        let array = list_builder.finish();
        Arc::new(array)
    }
}
