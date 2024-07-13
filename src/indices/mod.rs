use crate::types::err::*;
use crate::types::file;
use crate::types::record::*;
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyRow;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

mod ix_bruteforce;
mod type_algorithm;
mod type_distance;
mod type_filter;

pub use ix_bruteforce::IndexBruteForce;
pub use type_algorithm::IndexAlgorithm;
pub use type_distance::DistanceMetric;
pub use type_filter::*;

type TableName = String;

/// Type of SQL database used as a data source.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
pub enum SourceType {
    SQLITE,
    POSTGRES,
    MYSQL,
}

impl From<&str> for SourceType {
    fn from(value: &str) -> Self {
        match value {
            "sqlite" => SourceType::SQLITE,
            "postgres" | "postgresql" => SourceType::POSTGRES,
            "mysql" => SourceType::MYSQL,
            _ => panic!("Unsupported database scheme: {value}."),
        }
    }
}

/// Data source configuration for a vector index.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SourceConfig {
    /// Name of the SQL table to use as data source.
    pub table: TableName,
    /// Column name of the primary key in the data source.
    pub primary_key: ColumnName,
    /// Column name storing the vector data.
    pub vector: ColumnName,
    /// Optional list of column names storing additional metadata.
    pub metadata: Option<Vec<ColumnName>>,
    /// Filter to apply to the SQL query using WHERE clause.
    pub filter: Option<String>,
}

impl SourceConfig {
    /// Creates a source configuration with mostly default values.
    /// - `primary_key`: Column name of the primary key in the data source.
    /// - `vector`: Column name storing the vector data.
    ///
    /// Default configuration:
    /// - No metadata columns.
    /// - No query filter.
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
            filter: None,
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

    /// Adds a filter to the source configuration.
    /// - `filter`: Filter string without the WHERE keyword.
    ///
    /// Example:
    /// ```text
    /// year > 2000 AND genre = 'action'
    /// ```
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        let filter: String = filter.into();
        self.filter = Some(filter.trim().to_string());
        self
    }

    /// Returns the list of columns in the source configuration.
    pub fn columns(&self) -> Vec<ColumnName> {
        let mut columns = vec![&self.primary_key, &self.vector];
        if let Some(metadata) = &self.metadata {
            columns.extend(metadata.iter());
        }

        columns.into_iter().map(|s| s.to_string()).collect()
    }

    /// Generates a SQL query string based on the configuration.
    ///
    /// Example:
    /// ```sql
    /// SELECT id, vector, metadata
    /// FROM vectors
    /// WHERE metadata > 2000
    /// ```
    pub(crate) fn to_query(&self) -> String {
        let table = &self.table;
        let columns = self.columns().join(", ");
        let filter = match &self.filter {
            Some(filter) => format!("WHERE {}", filter),
            None => String::new(),
        };

        let query = format!("SELECT {columns} FROM {table} {filter}");
        query.trim().to_string()
    }

    /// Generates a SQL query string based on the configuration and checkpoint.
    /// Instead of returning a query to fetch all records, this method returns
    /// a query to fetch records from a specific RecordID.
    /// - `checkpoint`: Record ID to start the query from.
    pub(crate) fn to_query_after(&self, checkpoint: &RecordID) -> String {
        let table = &self.table;
        let columns = self.columns().join(", ");

        let mut filter = format!("WHERE id > {}", checkpoint.0);
        if let Some(string) = &self.filter {
            filter.push_str(&format!(" AND ({string})"));
        }

        let query = format!("SELECT {columns} FROM {table} {filter}");
        query.trim().to_string()
    }

    /// Creates a tuple of record ID and record data from a row.
    pub(crate) fn to_record(
        &self,
        row: &AnyRow,
    ) -> Result<(RecordID, Record), Error> {
        let id = RecordID::from_row(&self.primary_key, row)?;
        let vector = Vector::from_row(&self.vector, row)?;

        let mut metadata = HashMap::new();
        if let Some(metadata_columns) = &self.metadata {
            for column in metadata_columns {
                let value = RowOps::from_row(column, row)?;
                metadata.insert(column.to_owned(), value);
            }
        }

        let record = Record { vector, data: metadata };
        Ok((id, record))
    }
}

/// Metadata about the index for operations and optimizations.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IndexMetadata {
    /// Hidden records that will not be included in search results.
    pub hidden: Vec<RecordID>,
    /// Last inserted data reference used for incremental insertion.
    pub last_inserted: Option<RecordID>,
    /// Number of records in the index.
    pub count: usize,
}

/// Nearest neighbor search result.
#[derive(Debug)]
pub struct SearchResult {
    /// ID of the record in the data source.
    pub id: RecordID,
    /// Record metadata.
    pub data: HashMap<ColumnName, Option<RecordData>>,
    /// Distance between the query and the record.
    pub distance: f32,
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}

/// Trait for initializing a new index implementation.
///
/// This will be used by the IndexAlgorithm enum to initialize a new index
/// based on the algorithm and configuration. In addition to this trait,
/// the index struct should implement the VectorIndex trait.
pub trait IndexOps: Debug + Serialize + DeserializeOwned {
    /// Initializes an empty index with the given configuration.
    fn new(config: SourceConfig, metric: DistanceMetric) -> Self;

    /// Reads and deserializes the index from a file.
    fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        file::read_binary_file(path)
    }

    /// Serializes and persists the index to a file.
    fn persist(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        file::write_binary_file(path, self)
    }
}

/// Trait for vector index implementations.
///
/// For each index algorithm, a separate struct and implementation
/// of this trait is required. Roughly, the index struct should look like:
///
/// ```text
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Index{{ Algorithm }} {
///     config: SourceConfig,
///     metric: DistanceMetric,
///     metadata: IndexMetadata,
///     data: HashMap<RecordID, Record>,
///     // Other fields...
/// }
/// ```
pub trait VectorIndex: Debug {
    /// Returns the configuration of the index.
    fn config(&self) -> &SourceConfig;

    /// Returns the distance metric used by the index.
    fn metric(&self) -> &DistanceMetric;

    /// Returns metadata about the index.
    fn metadata(&self) -> &IndexMetadata;

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

    /// Searches for the nearest neighbors based on the query vector.
    /// - `query`: Query vector.
    /// - `k`: Number of nearest neighbors to return.
    fn search(
        &self,
        query: Vector,
        k: usize,
    ) -> Result<Vec<SearchResult>, Error>;

    /// Searches the nearest neighbors based on the query vector and filters.
    /// - `query`: Query vector.
    /// - `k`: Number of nearest neighbors to return.
    /// - `filters`: Filters to apply to the search results.
    fn search_with_filters(
        &self,
        query: Vector,
        k: usize,
        filters: Filters,
    ) -> Result<Vec<SearchResult>, Error>;

    /// Hides certain records from the search result permanently.
    fn hide(&mut self, record_ids: Vec<RecordID>) -> Result<(), Error>;

    /// Returns the index as Any type for dynamic casting. This method
    /// allows the index to be downcast to a specific index struct to
    /// be serialized and stored in a file.
    fn as_any(&self) -> &dyn Any;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_config_new() {
        let config = SourceConfig::new("table", "id", "embedding");
        let query = config.to_query();
        assert_eq!(query, "SELECT id, embedding FROM table");
    }

    #[test]
    fn test_source_config_new_complete() {
        let config = SourceConfig::new("table", "id", "embedding")
            .with_metadata(vec!["metadata"])
            .with_filter("id > 100");

        let query = config.to_query();
        let expected =
            "SELECT id, embedding, metadata FROM table WHERE id > 100";
        assert_eq!(query, expected);
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;

    pub fn populate_index(index: &mut impl VectorIndex) {
        let mut records = HashMap::new();
        for i in 0..100 {
            let id = RecordID(i as u32);
            let vector = Vector::from(vec![i as f32; 128]);
            let data = HashMap::from([(
                "number".to_string(),
                Some(RecordData::Integer(1000 + i)),
            )]);

            let record = Record { vector, data };
            records.insert(id, record);
        }

        index.fit(records).unwrap();
    }

    pub fn test_search(index: &impl VectorIndex) {
        let query = Vector::from(vec![0.0; 128]);
        let k = 10;
        let results = index.search(query, k).unwrap();

        assert_eq!(results.len(), k);
        assert_eq!(results[0].id, RecordID(0));
        assert_eq!(results[0].distance, 0.0);
        assert_eq!(results[9].id, RecordID(9));
    }

    pub fn test_search_with_filters(index: &impl VectorIndex) {
        let query = Vector::from(vec![0.0; 128]);
        let k = 10;
        let filters = Filters::from("number > 1010");
        let results = index.search_with_filters(query, k, filters).unwrap();

        assert_eq!(results.len(), k);
        assert_eq!(results[0].id, RecordID(11));
    }
}
