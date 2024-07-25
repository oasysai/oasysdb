use crate::types::distance::DistanceMetric;
use crate::types::err::{Error, ErrorCode};
use crate::types::filter::*;
use crate::types::record::*;
use crate::utils::file;
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyRow;
use std::any::Any;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Debug;
use std::path::Path;

mod idx_flat;
mod idx_ivfpq;

// Re-export indices and their parameter types.
pub use idx_flat::{IndexFlat, ParamsFlat};
pub use idx_ivfpq::{IndexIVFPQ, ParamsIVFPQ};

/// Name of the SQL table to use as a data source.
pub type TableName = String;

/// Type of SQL database used as a data source.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceType {
    SQLITE,
    POSTGRES,
    MYSQL,
}

impl From<&str> for SourceType {
    /// Converts source URL scheme to a source type.
    fn from(scheme: &str) -> Self {
        match scheme {
            "sqlite" => SourceType::SQLITE,
            "postgres" | "postgresql" => SourceType::POSTGRES,
            "mysql" => SourceType::MYSQL,
            _ => panic!("Unsupported database scheme: {scheme}."),
        }
    }
}

/// Data source configuration for a vector index.
///
/// The column data types used as the data source must be the following:
/// - Primary Key: Unique auto-incremented integer.
/// - Vector: Array of floats stored as JSON string or binary.
/// - Metadata: Primitive types like string, integer, float, or boolean.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Name of the SQL table to use as data source.
    pub table: TableName,
    /// Column name of the primary key in the source table.
    pub primary_key: ColumnName,
    /// Name of the column storing the vector data.
    pub vector: ColumnName,
    /// Optional list of column names of additional metadata.
    pub metadata: Option<Vec<ColumnName>>,
    /// Filter to apply to the SQL query using WHERE clause.
    pub filter: Option<String>,
}

#[cfg(test)]
impl Default for SourceConfig {
    fn default() -> Self {
        SourceConfig {
            table: "embeddings".into(),
            primary_key: "id".into(),
            vector: "vector".into(),
            metadata: None,
            filter: None,
        }
    }
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
        table: impl Into<TableName>,
        primary_key: impl Into<ColumnName>,
        vector: impl Into<ColumnName>,
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
    /// OasysDB only supports primitive data types for metadata such as:
    /// - String
    /// - Integer
    /// - Float
    /// - Boolean
    pub fn with_metadata(
        mut self,
        metadata: Vec<impl Into<ColumnName>>,
    ) -> Self {
        self.metadata = Some(metadata.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Adds a filter to the source configuration.
    /// - `filter`: SQL filter string without the WHERE keyword.
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

    /// Returns the list of columns in the following order:
    /// - Primary Key
    /// - Vector
    /// - Metadata (if any)
    pub fn columns(&self) -> Vec<ColumnName> {
        let mut columns = vec![&self.primary_key, &self.vector];
        if let Some(metadata) = &self.metadata {
            columns.extend(metadata.iter());
        }

        columns.into_iter().map(|s| s.to_owned()).collect()
    }

    /// Generates a SQL query based on the source configuration.
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

    /// Generates a SQL query string based on the configuration and a primary
    /// key checkpoint. Instead of returning a query to fetch all records,
    /// this method returns a query to fetch records from a specific RecordID.
    /// - `checkpoint`: Record ID to start the query from.
    pub(crate) fn to_query_after(&self, checkpoint: &RecordID) -> String {
        let table = &self.table;
        let pk = &self.primary_key;
        let columns = self.columns().join(", ");

        // Prioritize the primary key filtering before
        // joining with the optional filter.
        let mut filter = format!("WHERE {pk} > {}", checkpoint.0);
        if let Some(string) = &self.filter {
            filter.push_str(&format!(" AND ({string})"));
        }

        let query = format!("SELECT {columns} FROM {table} {filter}");
        query.trim().to_string()
    }

    /// Creates a tuple of record ID and record data from a row.
    /// - `row`: SQL row containing the record data.
    pub(crate) fn to_record(
        &self,
        row: &AnyRow,
    ) -> Result<(RecordID, Record), Error> {
        let id = RecordID::from_row(&self.primary_key, row)?;
        let vector = Vector::from_row(&self.vector, row)?;

        // Parse all metadata from the row if any.
        let mut metadata = HashMap::new();
        if let Some(metadata_columns) = &self.metadata {
            for column in metadata_columns {
                let value = RowOps::from_row(column.to_owned(), row)?;
                metadata.insert(column.to_owned(), value);
            }
        }

        let record = Record { vector, data: metadata };
        Ok((id, record))
    }
}

/// Algorithm options used to index and search vectors.
///
/// You might want to use a different algorithm based on the size
/// of the data and the desired search performance. For example,
/// the Flat algorithm is gives good performance and recall for small datasets.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexAlgorithm {
    Flat(ParamsFlat),   // -> IndexFlat
    IVFPQ(ParamsIVFPQ), // -> IndexIVFPQ
}

impl IndexAlgorithm {
    /// Returns the name of the algorithm in uppercase.
    pub fn name(&self) -> &str {
        match self {
            Self::Flat(_) => "FLAT",
            Self::IVFPQ(_) => "IVFPQ",
        }
    }
}

impl PartialEq for IndexAlgorithm {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for IndexAlgorithm {}

impl IndexAlgorithm {
    /// Initializes a new index based on the algorithm and its parameters.
    pub(crate) fn initialize(&self) -> Result<Box<dyn VectorIndex>, Error> {
        macro_rules! initialize {
            ($index_type:ident, $params:expr) => {{
                let index = $index_type::new($params)?;
                Ok(Box::new(index))
            }};
        }

        match self.to_owned() {
            Self::Flat(params) => initialize!(IndexFlat, params),
            Self::IVFPQ(params) => initialize!(IndexIVFPQ, params),
        }
    }

    /// Loads an index from a file based on the algorithm.
    /// - `path`: Path to the file where the index is stored.
    pub(crate) fn load_index(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Box<dyn VectorIndex>, Error> {
        macro_rules! load {
            ($index_type:ident) => {{
                let index = Self::_load_index::<$index_type>(path)?;
                Ok(Box::new(index))
            }};
        }

        match self {
            Self::Flat(_) => load!(IndexFlat),
            Self::IVFPQ(_) => load!(IndexIVFPQ),
        }
    }

    /// Persists the index to a file based on the algorithm.
    /// - `path`: Path to the file where the index will be stored.
    /// - `index`: Index to persist as a trait object.
    pub(crate) fn persist_index(
        &self,
        path: impl AsRef<Path>,
        index: &dyn VectorIndex,
    ) -> Result<(), Error> {
        macro_rules! persist {
            ($index_type:ident) => {{
                Self::_persist_index::<$index_type>(path, index)
            }};
        }

        match self {
            Self::Flat(_) => persist!(IndexFlat),
            Self::IVFPQ(_) => persist!(IndexIVFPQ),
        }
    }

    fn _load_index<T: VectorIndex + IndexOps + 'static>(
        path: impl AsRef<Path>,
    ) -> Result<T, Error> {
        let index = T::load(path)?;
        Ok(index)
    }

    fn _persist_index<T: VectorIndex + IndexOps + 'static>(
        path: impl AsRef<Path>,
        index: &dyn VectorIndex,
    ) -> Result<(), Error> {
        let index = index.as_any().downcast_ref::<T>().ok_or_else(|| {
            let code = ErrorCode::InternalError;
            let message = "Failed to downcast index to concrete type.";
            Error::new(code, message)
        })?;

        index.persist(path)?;
        Ok(())
    }
}

/// Metadata about the index operations.
///
/// This information should be available to all index implementations
/// to keep track of the overall state of the index. This data is useful
/// to optimize the index operations and to provide insights about the index.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IndexMetadata {
    /// Status whether the index has been built or not.
    pub built: bool,
    /// Last inserted record reference used for incremental insertion.
    pub last_inserted: Option<RecordID>,
}

/// Nearest neighbor search result.
///
/// This struct contains the additional metadata of the records
/// which is often used for post-search operations such as using
/// the metadata as a context for RAG (Retrieval Augmented Generation).
#[derive(Debug)]
pub struct SearchResult {
    /// ID of the record in the data source.
    pub id: RecordID,
    /// Record metadata.
    pub data: HashMap<ColumnName, Option<DataValue>>,
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.partial_cmp(&other.distance).unwrap_or(Ordering::Equal)
    }
}

/// Trait for an index implementation.
///
/// This trait defines the basic operations that an index should support.
/// The trait comes with default implementations for loading and persisting
/// the index to a file that should work for most cases.
pub trait IndexOps: Debug + Serialize + DeserializeOwned {
    /// Initializes an empty index with the given parameters.
    /// - `params`: Index specific parameters.
    fn new(params: impl IndexParams) -> Result<Self, Error>;

    /// Reads and deserializes the index from a file.
    fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        file::read_binary_file(path)
    }

    /// Serializes and persists the index to a file.
    fn persist(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        file::write_binary_file(path, self)
    }
}

/// Trait for operating an index implementation.
///
/// This trait defines operational methods to interact with the index such as
/// fitting and searching the index. Every index implementation should have the
/// following fields:
///
/// - `params`: Index-specific parameters.
/// - `metadata`: Index metadata.
/// - `data`: Records stored in the index.
pub trait VectorIndex: Debug + Send + Sync {
    /// Returns the distance metric used by the index.
    fn metric(&self) -> &DistanceMetric;

    /// Returns metadata about the index.
    fn metadata(&self) -> &IndexMetadata;

    /// Builds the index from scratch based on the records.
    /// - `records`: Records to build the index on.
    fn build(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error>;

    /// Inserts new records into the index incrementally.
    /// - `records`: Records to insert into the index.
    fn insert(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error>;

    /// Deletes records from the index data store.
    /// - `ids`: List of record IDs to delete from the index.
    fn delete(&mut self, ids: Vec<RecordID>) -> Result<(), Error>;

    /// Searches for the nearest neighbors of the query vector.
    /// - `query`: Query vector.
    /// - `k`: Number of nearest neighbors to return.
    /// - `filters`: Filters to apply to the search results.
    ///
    /// Returns search results sorted by their distance to the query.
    /// The degree of the distance might vary depending on the metric
    /// used but the smallest distance always means the most relevant
    /// record to the query.
    fn search(
        &self,
        query: Vector,
        k: usize,
        filters: Filters,
    ) -> Result<Vec<SearchResult>, Error>;

    /// Returns the number of records in the index.
    fn len(&self) -> usize;

    /// Checks if the index has no records.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the index as Any type for dynamic casting.
    ///
    /// This method allows the index trait object to be downcast to a
    /// specific index struct to be serialized and stored in a file.
    fn as_any(&self) -> &dyn Any;
}

/// Trait for custom index parameters.
///
/// Every index implementation should have a custom parameter struct that
/// implements this trait. The parameters struct should also derive the
/// Serialize and Deserialize traits as it will be stored inside of the index.
pub trait IndexParams: Debug + Default + Clone {
    /// Returns the distance metric set in the parameters.
    fn metric(&self) -> &DistanceMetric;

    /// Returns the parameters as Any type for dynamic casting.
    fn as_any(&self) -> &dyn Any;
}

/// Downcasts the index parameters trait object to a concrete type.
/// - `params`: Index parameters trait object.
pub(crate) fn downcast_params<T: IndexParams + 'static>(
    params: impl IndexParams,
) -> Result<T, Error> {
    params.as_any().downcast_ref::<T>().cloned().ok_or_else(|| {
        let code = ErrorCode::InternalError;
        let message = "Failed to downcast index parameters to concrete type.";
        Error::new(code, message)
    })
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
                "number".into(),
                Some(DataValue::Integer(1000 + i)),
            )]);

            let record = Record { vector, data };
            records.insert(id, record);
        }

        index.build(records).unwrap();
        assert_eq!(index.len(), 100);
    }

    pub fn test_basic_search(index: &impl VectorIndex) {
        let query = Vector::from(vec![0.0; 128]);
        let k = 10;
        let results: Vec<RecordID> = index
            .search(query, k, Filters::NONE)
            .unwrap()
            .iter()
            .map(|result| result.id)
            .collect();

        assert_eq!(results.len(), k);
        for i in 0..k {
            assert!(results.contains(&RecordID(i as u32)));
        }
    }

    pub fn test_advanced_search(index: &impl VectorIndex) {
        let query = Vector::from(vec![0.0; 128]);
        let k = 10;
        let filters = Filters::from("number > 1010");
        let results = index.search(query, k, filters).unwrap();

        assert_eq!(results.len(), k);
        assert_eq!(results[0].id, RecordID(11));
    }
}
