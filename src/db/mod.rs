use crate::types::*;
use arrow::datatypes::DataType;
use arrow_schema::{Field, Fields, Schema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, RwLock as Lock};
use tonic::{Request, Response, Status};

mod collection;
mod database;
mod database_service;

pub use collection::*;
pub use database::*;

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
