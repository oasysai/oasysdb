use super::*;
use futures::executor;
use futures::stream::StreamExt;
use sqlx::any::install_default_drivers;
use sqlx::Acquire;
use sqlx::Executor;
use url::Url;
use uuid::Uuid;

type DatabaseURL = String;
type IndexName = String;
type IndexFile = PathBuf;

/// The vector database interface.
pub struct Database {
    root: PathBuf,
    state: DatabaseState,
    conn: SourceConnection,
}

impl Database {
    /// Opens existing or creates a new vector database.
    /// - `root`: Root directory of the database.
    /// - `source_url`: URL to SQL database.
    ///
    /// This will attempt to restore the database state from the file first.
    /// If the file does not exist, it will create a new database.
    /// When creating a new database, a data source is required.
    ///
    /// Source URL examples:
    /// ```txt
    /// sqlite://sqlite.db
    /// mysql://user:password@localhost:3306/db
    /// postgresql://user:password@localhost:5432/db
    /// ```
    pub fn open(
        root: impl Into<PathBuf>,
        source_url: Option<impl Into<String>>,
    ) -> Result<Database, Error> {
        let root_dir: PathBuf = root.into();
        let indices_dir = root_dir.join("indices");
        if !indices_dir.try_exists()? {
            fs::create_dir_all(&indices_dir)?;
        }

        let state_file = root_dir.join("odbstate");
        let state = if state_file.try_exists()? {
            file::read_binary_file(state_file)?
        } else {
            let source = source_url.ok_or_else(|| {
                let code = ErrorCode::MissingSource;
                let message = "Data source is required to create a database.";
                Error::new(code, message)
            })?;

            let source: String = source.into();
            DatabaseState::validate_source(&source)?;

            let state = DatabaseState { source, indices: HashMap::new() };
            file::write_binary_file(state_file, &state)?;
            state
        };

        let conn: SourceConnection = state.connect()?;
        Ok(Self { root: root_dir, state, conn })
    }

    /// Creates a new index in the database asynchronously.
    /// - `name`: Name of the index.
    /// - `algorithm`: Indexing algorithm to use.
    /// - `metric`: Distance metric for the index.
    /// - `config`: Index data source configuration.
    pub async fn async_create_index(
        &mut self,
        name: impl Into<String>,
        algorithm: IndexAlgorithm,
        metric: DistanceMetric,
        config: SourceConfig,
    ) -> Result<(), Error> {
        let state_file = self.state_file();

        // Create a new file where the index will be stored.
        let index_file = {
            let uuid = Uuid::new_v4().to_string();
            self.indices_dir().join(uuid)
        };

        let query = config.to_query();
        let conn = self.conn.acquire().await?;
        let mut stream = sqlx::query(&query).fetch(conn);

        let mut records = HashMap::new();
        while let Some(row) = stream.next().await {
            let row = row?;
            let (id, record) = config.to_record(&row)?;
            records.insert(id, record);
        }

        let mut index = algorithm.initialize(config, metric);
        index.fit(records)?;

        // Persist the index to the file.
        algorithm.persist_index(&index_file, index)?;

        // Update db state with the new index.
        let index_ref = IndexRef { algorithm, file: index_file.clone() };
        self.state.indices.insert(name.into(), index_ref);
        file::write_binary_file(&state_file, &self.state)?;

        Ok(())
    }

    /// Creates a new index in the database synchronously.
    pub fn create_index(
        &mut self,
        name: impl Into<String>,
        algorithm: IndexAlgorithm,
        metric: DistanceMetric,
        config: SourceConfig,
    ) -> Result<(), Error> {
        executor::block_on(
            self.async_create_index(name, algorithm, metric, config),
        )
    }

    /// Returns an index reference by name.
    ///
    /// This method is useful for deserializing and accessing
    /// the index directly from the file based on the algorithm type.
    pub fn get_index_ref(&self, name: impl AsRef<str>) -> Option<&IndexRef> {
        self.state.indices.get(name.as_ref())
    }

    /// Retrieves an index from the file and returns it as a trait object.
    /// - `name`: Index name.
    pub fn get_index(
        &self,
        name: impl AsRef<str>,
    ) -> Option<Box<dyn VectorIndex>> {
        let IndexRef { algorithm, file } = self.get_index_ref(name)?;
        algorithm.load_index(file).ok()
    }

    /// Retrieves an index from the file and if found, returns it as a
    /// trait object. Otherwise, returns a not found error.
    /// - `name`: Index name.
    pub fn try_get_index(
        &self,
        name: impl AsRef<str>,
    ) -> Result<Box<dyn VectorIndex>, Error> {
        let name = name.as_ref();
        self.get_index(name).ok_or_else(|| {
            let code = ErrorCode::NotFound;
            let message = format!("Index not found in database: {name}.");
            Error::new(code, message)
        })
    }

    /// Updates the index with new records from the source asynchronously.
    /// - `name`: Index name.
    ///
    /// This method checks the index for the last inserted record and queries
    /// the source database for new records after that checkpoint. It then
    /// updates the index with the new records.
    pub async fn async_refresh_index(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<(), Error> {
        let name = name.as_ref();
        let index_ref = self.get_index_ref(name).ok_or_else(|| {
            let code = ErrorCode::NotFound;
            let message = format!("Index not found: {name}.");
            Error::new(code, message)
        })?;

        // Cloning is necessary here to avoid borrowing issues.
        let IndexRef { algorithm, file } = index_ref.to_owned();

        // It's safe to unwrap here because we validated that index exists by
        // calling get_index_ref method above.
        let mut index = self.get_index(name).unwrap();

        let query = {
            let meta = index.metadata();
            let checkpoint = meta.last_inserted.unwrap_or_default();
            index.config().to_query_after(&checkpoint)
        };

        let conn = self.conn.acquire().await?;
        let mut stream = sqlx::query(&query).fetch(conn);

        let mut records = HashMap::new();
        while let Some(row) = stream.next().await {
            let row = row?;
            let (id, record) = index.config().to_record(&row)?;
            records.insert(id, record);
        }

        index.fit(records)?;
        algorithm.persist_index(file, index)?;
        Ok(())
    }

    /// Updates the index with new records from the source synchronously.
    pub fn refresh_index(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<(), Error> {
        executor::block_on(self.async_refresh_index(name))
    }

    /// Searches the index for the nearest vectors to the query vector.
    /// - `name`: Index name.
    /// - `query`: Query vector.
    /// - `k`: Number of nearest neighbors to return.
    pub fn search_index(
        &self,
        name: impl AsRef<str>,
        query: impl Into<Vector>,
        k: usize,
    ) -> Result<Vec<SearchResult>, Error> {
        let index = self.try_get_index(name)?;
        index.search(query.into(), k)
    }

    /// Searches the index for nearest neighbors with post-search filters.
    /// - `name`: Index name.
    /// - `query`: Query vector.
    /// - `k`: Number of nearest neighbors to return.
    /// - `filters`: SQL-like filters to apply.
    pub fn search_index_with_filters(
        &self,
        name: impl AsRef<str>,
        query: impl Into<Vector>,
        k: usize,
        filters: impl Into<Filters>,
    ) -> Result<Vec<SearchResult>, Error> {
        let index = self.try_get_index(name)?;
        index.search_with_filters(query.into(), k, filters.into())
    }

    /// Rebuilds the index from the existing records in the index.
    /// - `name`: Index name.
    ///
    /// Some indexing algorithms may not support perfect incremental updates.
    /// This method can be useful to rebalance the index.
    pub fn rebuild_index(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<(), Error> {
        let name = name.as_ref();
        let mut index = self.try_get_index(name)?;
        index.refit()?;

        // Unwrap is safe here because we validated that the index exists above.
        let IndexRef { algorithm, file } = self.get_index_ref(name).unwrap();
        algorithm.persist_index(file, index)?;

        Ok(())
    }

    /// Deletes an index from the database given its name.
    pub fn delete_index(&mut self, name: impl AsRef<str>) -> Result<(), Error> {
        let name = name.as_ref();
        let index_ref = self.state.indices.remove(name).ok_or_else(|| {
            let code = ErrorCode::NotFound;
            let message = format!("Index doesn't exist: {name}.");
            Error::new(code, message)
        })?;

        fs::remove_file(index_ref.file())?;
        file::write_binary_file(self.state_file(), &self.state)
    }

    /// Returns the state object of the database.
    pub fn state(&self) -> &DatabaseState {
        &self.state
    }

    /// Persists the state of the database to the state file.
    pub fn persist_state(&self) -> Result<(), Error> {
        file::write_binary_file(self.state_file(), &self.state)
    }
}

// Write internal database methods here.
impl Database {
    fn state_file(&self) -> PathBuf {
        self.root.join("odbstate")
    }

    fn indices_dir(&self) -> PathBuf {
        self.root.join("indices")
    }

    #[allow(dead_code)]
    async fn async_execute_sql(
        &mut self,
        query: impl AsRef<str>,
    ) -> Result<(), Error> {
        let conn = self.conn.acquire().await?;
        conn.execute(query.as_ref()).await?;
        Ok(())
    }
}

/// The state of the vector database.
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseState {
    source: DatabaseURL,
    indices: HashMap<IndexName, IndexRef>,
}

impl DatabaseState {
    /// Connects to the source SQL database asynchronously.
    pub async fn async_connect(&self) -> Result<SourceConnection, Error> {
        install_default_drivers();
        Ok(SourceConnection::connect(&self.source).await?)
    }

    /// Connects to the source SQL database.
    pub fn connect(&self) -> Result<SourceConnection, Error> {
        executor::block_on(self.async_connect())
    }

    /// Returns the type of the source database.
    /// - sqlite
    /// - mysql
    /// - postgresql
    pub fn source_type(&self) -> SourceType {
        // We can safely unwrap here because
        // we have already validated the source URL.
        let url = self.source.parse::<Url>().unwrap();
        url.scheme().into()
    }

    /// Validates the data source URL.
    pub fn validate_source(url: impl Into<String>) -> Result<(), Error> {
        let url: String = url.into();
        let url = url.parse::<Url>().map_err(|_| {
            let code = ErrorCode::InvalidSource;
            let message = "Invalid database source URL.";
            Error::new(code, message)
        })?;

        let valid_schemes = ["sqlite", "mysql", "postgresql"];
        if !valid_schemes.contains(&url.scheme()) {
            let code = ErrorCode::InvalidSource;
            let message = format!(
                "Unsupported database scheme. Choose between: {}.",
                valid_schemes.join(", ")
            );

            return Err(Error::new(code, message));
        }

        Ok(())
    }
}

/// Details about the index and where it is stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRef {
    algorithm: IndexAlgorithm,
    file: IndexFile,
}

impl IndexRef {
    pub fn algorithm(&self) -> &IndexAlgorithm {
        &self.algorithm
    }

    pub fn file(&self) -> &IndexFile {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::RecordID;
    use sqlx::{Executor, Row};

    const TABLE: &str = "embeddings";
    const TEST_INDEX: &str = "test_index";

    #[test]
    fn test_database_open() {
        assert!(create_test_database().is_ok());
    }

    #[test]
    fn test_database_create_index() {
        let db = create_test_database().unwrap();

        let index = db.get_index(TEST_INDEX).unwrap();
        let metadata = index.metadata();

        assert_eq!(metadata.count, 100);
        assert_eq!(metadata.last_inserted, Some(RecordID(100)));
    }

    #[test]
    fn test_database_refresh_index() {
        let mut db = create_test_database().unwrap();
        let query = generate_insert_query(100, 10);
        executor::block_on(db.async_execute_sql(query)).unwrap();

        db.refresh_index(TEST_INDEX).unwrap();

        let index = db.get_index(TEST_INDEX).unwrap();
        let metadata = index.metadata();

        assert_eq!(metadata.count, 110);
        assert_eq!(metadata.last_inserted, Some(RecordID(110)));
    }

    #[test]
    fn test_database_search_index() {
        let db = create_test_database().unwrap();
        let query = vec![0.0; 128];
        let results = db.search_index(TEST_INDEX, query, 5).unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].id, RecordID(1));
        assert_eq!(results[0].distance, 0.0);
    }

    #[test]
    fn test_database_search_index_with_filters() {
        let db = create_test_database().unwrap();
        let query = vec![0.0; 128];
        let filters = Filters::from("data >= 1050");
        let results = db
            .search_index_with_filters(TEST_INDEX, query, 5, filters)
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].id, RecordID(51));
    }

    #[test]
    fn test_database_rebuild_index() {
        let mut db = create_test_database().unwrap();
        db.rebuild_index(TEST_INDEX).unwrap();

        let index = db.get_index(TEST_INDEX).unwrap();
        assert_eq!(index.metadata().count, 100);
    }

    #[test]
    fn test_database_delete_index() {
        let mut db = create_test_database().unwrap();
        db.delete_index(TEST_INDEX).unwrap();

        let state = db.state();
        assert!(!state.indices.contains_key(TEST_INDEX));
    }

    fn create_test_database() -> Result<Database, Error> {
        let path = PathBuf::from("odb_data");
        if path.try_exists().is_ok() {
            fs::remove_dir_all(&path)?;
        }

        let db_path = file::get_tmp_dir()?.join("sqlite.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let mut db = Database::open(path, Some(db_url.clone()))?;
        let state = db.state();
        assert_eq!(state.source_type(), SourceType::SQLITE);

        executor::block_on(setup_test_source(db_url))?;
        create_test_index(&mut db)?;
        Ok(db)
    }

    fn create_test_index(db: &mut Database) -> Result<(), Error> {
        let config = SourceConfig::new(TABLE, "id", "vector")
            .with_metadata(vec!["data"]);

        db.create_index(
            TEST_INDEX,
            IndexAlgorithm::BruteForce,
            DistanceMetric::Euclidean,
            config,
        )?;

        let index_ref = db.get_index_ref(TEST_INDEX).unwrap();
        assert_eq!(index_ref.algorithm(), &IndexAlgorithm::BruteForce);
        Ok(())
    }

    fn generate_insert_query(start: u8, count: u8) -> String {
        let start = start as u16;
        let end = start + count as u16;

        let mut values = vec![];
        for i in start..end {
            let vector = vec![i as f32; 128];
            let vector = serde_json::to_string(&vector).unwrap();
            let data = 1000 + i;
            values.push(format!("({vector:?}, {data})"));
        }

        let values = values.join(",\n");
        format!(
            "INSERT INTO {TABLE} (vector, data)
            VALUES {values}"
        )
    }

    async fn setup_test_source(url: impl Into<String>) -> Result<(), Error> {
        let url: String = url.into();
        let mut conn = SourceConnection::connect(&url).await?;

        let create_table = format!(
            "CREATE TABLE IF NOT EXISTS {TABLE} (
            id INTEGER PRIMARY KEY,
            vector JSON NOT NULL,
            data INTEGER NOT NULL)"
        );

        let insert_records = generate_insert_query(0, 100);

        conn.execute("DROP TABLE IF EXISTS embeddings").await?;
        conn.execute(create_table.as_str()).await?;
        conn.execute(insert_records.as_str()).await?;

        let count = conn
            .fetch_one("SELECT COUNT(*) FROM embeddings")
            .await?
            .get::<i64, usize>(0);

        assert_eq!(count, 100);
        Ok(())
    }
}
