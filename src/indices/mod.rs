use crate::types::err::Error;
use crate::types::record::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

mod ix_bruteforce;

pub use ix_bruteforce::IndexBruteForce;

type TableName = String;

/// Data source configuration for a vector index.
#[derive(Debug, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Name of the SQL table to use as data source.
    pub table: TableName,
    /// Column name of the primary key in the data source.
    pub primary_key: ColumnName,
    /// Column name storing the vector data.
    pub vector: ColumnName,
    /// Optional list of column names storing additional metadata.
    pub metadata: Option<Vec<ColumnName>>,
}

impl SourceConfig {
    /// Creates a source configuration with mostly default values.
    /// - `primary_key`: Column name of the primary key in the data source.
    /// - `vector`: Column name storing the vector data.
    ///
    /// Default configuration:
    /// - No metadata columns.
    pub fn new(
        table: impl Into<String>,
        primary_key: impl Into<String>,
        vector: impl Into<String>,
    ) -> Self {
        SourceConfig {
            table: table.into(),
            primary_key: primary_key.into(),
            vector: vector.into(),
            metadata: None,
        }
    }

    /// Adds a list of metadata columns to the source configuration.
    /// - `metadata`: List of metadata column names.
    ///
    /// OasysDB only supports primitive data types for metadata columns such as:
    /// - String
    /// - Integer
    /// - Float
    /// - Boolean
    pub fn with_metadata(mut self, metadata: Vec<impl Into<String>>) -> Self {
        self.metadata = Some(metadata.into_iter().map(|s| s.into()).collect());
        self
    }
}

/// Algorithm options used to index and search vectors.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum IndexAlgorithm {
    BruteForce,
}

/// Trait for vector index implementations.
///
/// For each index algorithm, a separate struct and implementation
/// of this trait is required. Also, these are some fields that
/// should be included in the Index struct:
///
/// ```text
/// struct Index{{ Algorithm }} {
///     config: SourceConfig,
///     data: HashMap<RecordID, Record>,
///     hidden: Vec<RecordID>,
///     // Other fields...
/// }
/// ```
pub trait VectorIndex: Debug + Serialize + DeserializeOwned {
    /// Returns the configuration of the index.
    fn config(&self) -> &SourceConfig;

    /// Returns the record IDs hidden from the result.
    fn hidden(&self) -> &[RecordID];

    /// Initializes the index.
    fn new(source: SourceConfig) -> Self;

    /// Trains the index based on the new records.
    ///
    /// If the index has been trained and not empty, this method
    /// will incrementally train the index based on the current fitting.
    /// Otherwise, this method will train the index from scratch like normal.
    fn fit(&mut self, records: HashMap<RecordID, Record>) -> Result<(), Error>;

    /// Resets the index and re-trains it on the non-hidden records.
    ///
    /// Incremental fitting is not as optimal as fitting from scratch for
    /// some indexing algorithms. This method could be useful to re-balance
    /// the index after a certain threshold of incremental fitting.
    fn refit(&mut self) -> Result<(), Error>;

    /// Hides certain records from the search result permanently.
    fn hide(&mut self, record_ids: Vec<RecordID>) -> Result<(), Error>;
}
