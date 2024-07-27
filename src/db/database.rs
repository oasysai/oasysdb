use super::*;
use futures::executor;
use futures::stream::StreamExt;
use sqlx::any::install_default_drivers;
use sqlx::Acquire;
use std::sync::{Arc, Mutex};
use url::Url;
use uuid::Uuid;

// Type aliases for better readability.
type DatabaseURL = String;
type IndexName = String;
type IndexFile = PathBuf;
type Index = Arc<Mutex<Box<dyn VectorIndex>>>;
type IndicesPool = Mutex<HashMap<IndexName, Index>>;

/// The vector database interface.
///
/// The database is responsible for managing:
/// - Data flow between the source database and the indices.
/// - High-level indices operation and management.
/// - Persistance, retrieval, and in-memory pool of vector indices.
pub struct Database {
    root: PathBuf,
    state: Mutex<DatabaseState>,
    pool: IndicesPool,
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
        source_url: Option<impl Into<DatabaseURL>>,
    ) -> Result<Database, Error> {
        let root_dir: PathBuf = root.into();
        let indices_dir = root_dir.join("indices");
        if !indices_dir.try_exists()? {
            // Creating the indices directory will also create
            // the root directory if it doesn't exist.
            fs::create_dir_all(&indices_dir)?;
        }

        let state_file = root_dir.join("odbstate");
        let state = if state_file.try_exists()? {
            let mut state = DatabaseState::restore(&state_file)?;

            // If the source URL is provided, update the state.
            // This is useful in case the source URL has changed.
            if let Some(source) = source_url {
                state.with_source(source)?;
            }

            state
        } else {
            let source = source_url.ok_or_else(|| {
                let code = ErrorCode::MissingSource;
                let message = "Data source is required for a new database.";
                Error::new(code, message)
            })?;

            let indices = HashMap::new();
            let source = source.into();
            DatabaseState::validate_source(&source)?;

            // Persist the new state to the state file.
            let state = DatabaseState { source, indices };
            file::write_binary_file(state_file, &state)?;
            state
        };

        state.validate_connection()?;
        let state = Mutex::new(state);
        let pool: IndicesPool = Mutex::new(HashMap::new());
        Ok(Self { root: root_dir, state, pool })
    }

    /// Creates a new index in the database asynchronously.
    /// - `name`: Name of the index.
    /// - `algorithm`: Indexing algorithm to use.
    /// - `config`: Index data source configuration.
    pub async fn async_create_index(
        &self,
        name: impl Into<IndexName>,
        algorithm: IndexAlgorithm,
        config: SourceConfig,
    ) -> Result<(), Error> {
        // Query the source database for records.
        let query = config.to_query();
        let mut conn = self.state()?.async_connect().await?;
        let mut stream = sqlx::query(&query).fetch(conn.acquire().await?);

        // Process the rows from the query as records.
        let mut records = HashMap::new();
        while let Some(row) = stream.next().await {
            let row = row?;
            let (id, record) = config.to_record(&row)?;
            records.insert(id, record);
        }

        let index_name: IndexName = name.into();
        let index_file = {
            let uuid = Uuid::new_v4().to_string();
            self.indices_dir().join(uuid)
        };

        let mut index = algorithm.initialize()?;
        index.build(records)?;

        // Persist the index to a file.
        algorithm.persist_index(&index_file, index.as_ref())?;

        // Insert the index into the pool for easy access.
        {
            let mut pool = self.pool.lock()?;
            pool.insert(index_name.clone(), Arc::new(Mutex::new(index)));
        }

        // Update db state with the new index.
        // This closure is necessary to  make sure the lock is dropped
        // before persisting the state to the file.
        {
            let mut state = self.state.lock()?;
            let index_ref = IndexRef { algorithm, config, file: index_file };
            state.indices.insert(index_name, index_ref);
        }

        self.persist_state()?;
        Ok(())
    }

    /// Creates a new index in the database synchronously.
    /// - `name`: Name of the index.
    /// - `algorithm`: Indexing algorithm to use.
    /// - `config`: Index data source configuration.
    pub fn create_index(
        &self,
        name: impl Into<IndexName>,
        algorithm: IndexAlgorithm,
        config: SourceConfig,
    ) -> Result<(), Error> {
        executor::block_on(self.async_create_index(name, algorithm, config))
    }

    /// Returns an index reference.
    /// - `name`: Index name.
    ///
    /// This method can be used to deserialize the index directly from
    /// the file and load it into memory as an index object.
    pub fn get_index_ref(&self, name: impl AsRef<str>) -> Option<IndexRef> {
        let state = self.state.lock().ok()?;
        let index_ref = state.indices.get(name.as_ref())?;
        Some(index_ref.to_owned())
    }

    /// Retrieves an index and returns it as a trait object.
    /// - `name`: Index name.
    ///
    /// This method will return the index from the pool if it exists.
    /// Otherwise, it will load the index from the file and store it
    /// in the pool for future access.
    pub fn get_index(&self, name: impl AsRef<str>) -> Option<Index> {
        let name = name.as_ref();
        let IndexRef { algorithm, file, .. } = self.get_index_ref(name)?;

        // If the index is already in the indices pool, return it.
        let mut pool = self.pool.lock().ok()?;
        if let Some(index) = pool.get(name).cloned() {
            return Some(index);
        }

        // Load the index from the file and store it in the pool.
        // Then, return the index as a trait object.
        let index = algorithm.load_index(file).ok()?;
        let index: Index = Arc::new(Mutex::new(index));
        pool.insert(name.into(), index.clone());
        Some(index)
    }

    /// Retrieves an index and returns it in a result.
    /// - `name`: Index name.
    pub fn try_get_index(&self, name: impl AsRef<str>) -> Result<Index, Error> {
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
        &self,
        name: impl AsRef<str>,
    ) -> Result<(), Error> {
        let name = name.as_ref();
        let index_ref = self.get_index_ref(name).ok_or_else(|| {
            let code = ErrorCode::NotFound;
            let message = format!("Index not found: {name}.");
            Error::new(code, message)
        })?;

        // Cloning is necessary here to avoid borrowing issues.
        let IndexRef { algorithm, file, config } = index_ref.to_owned();

        // It's safe to unwrap here because we validated that index exists by
        // calling get_index_ref method above.
        let index: Index = self.get_index(name).unwrap();

        let (query, config) = {
            // We wrap the index lock in a closure to make sure it's dropped
            // before async functionalities are called.
            let index = index.lock()?;
            let meta = index.metadata();
            let checkpoint = meta.last_inserted.unwrap_or_default();
            (config.to_query_after(&checkpoint), config)
        };

        let mut conn = self.state()?.async_connect().await?;
        let mut stream = sqlx::query(&query).fetch(conn.acquire().await?);

        // Process the rows from the database as records.
        let mut records = HashMap::new();
        while let Some(row) = stream.next().await {
            let row = row?;
            let (id, record) = config.to_record(&row)?;
            records.insert(id, record);
        }

        // Update the index with new records and persist it.
        // We might want to persist the index after every fit operation.
        let mut index = index.lock()?;
        index.insert(records)?;
        algorithm.persist_index(file, index.as_ref())?;
        Ok(())
    }

    /// Updates the index with new records from the source synchronously.
    /// - `name`: Index name.
    pub fn refresh_index(&self, name: impl AsRef<str>) -> Result<(), Error> {
        executor::block_on(self.async_refresh_index(name))
    }

    /// Searches the index for nearest neighbors.
    /// - `name`: Index name.
    /// - `query`: Query vector.
    /// - `k`: Number of nearest neighbors to return.
    /// - `filters`: SQL-like filters to apply.
    ///
    /// The performance of this method depends on the indexing
    /// algorithm used when creating the index. ANNS algorithms
    /// may not return the exact nearest neighbors but perform
    /// much faster than linear search.
    pub fn search_index(
        &self,
        name: impl AsRef<str>,
        query: impl Into<Vector>,
        k: usize,
        filters: impl Into<Filters>,
    ) -> Result<Vec<SearchResult>, Error> {
        let index: Index = self.try_get_index(name)?;
        let index = index.lock()?;
        index.search(query.into(), k, filters.into())
    }

    /// Deletes an index from the database.
    /// - `name`: Index name.
    ///
    /// This method will remove the index from the pool and delete
    /// the index file from the disk. Returns an error if the index
    /// doesn't exist in the database.
    pub fn delete_index(&self, name: impl AsRef<str>) -> Result<(), Error> {
        let name = name.as_ref();
        let index_ref = {
            let mut state = self.state.lock()?;
            state.indices.remove(name).ok_or_else(|| {
                let code = ErrorCode::NotFound;
                let message = format!("Index doesn't exist: {name}.");
                Error::new(code, message)
            })?
        };

        self.release_indices(vec![name])?;
        fs::remove_file(index_ref.file())?;
        self.persist_state()
    }

    /// Loads indices to the pool if they are not already loaded.
    /// - `names`: Names of the indices.
    pub fn load_indices(
        &self,
        names: Vec<impl AsRef<str>>,
    ) -> Result<(), Error> {
        let state = self.state()?;
        if names.iter().any(|name| !state.indices.contains_key(name.as_ref())) {
            let code = ErrorCode::NotFound;
            let message = "Some indices are not found in the database.";
            return Err(Error::new(code, message));
        }

        // Using the get_index method to avoid code duplication.
        for name in names {
            self.get_index(name);
        }

        Ok(())
    }

    /// Releases indices from the pool.
    /// - `names`: Names of the indices.
    ///
    /// This method can free up memory by removing indices from the pool.
    /// After the indices are released, when they need to be accessed again,
    /// they will be loaded from the file.
    ///
    /// Loading indices from the file might take some time. Therefore,
    /// it's recommended to keep the frequently used indices in the pool.
    pub fn release_indices(
        &self,
        names: Vec<impl AsRef<str>>,
    ) -> Result<(), Error> {
        let mut pool = self.pool.lock()?;
        for name in names {
            let name = name.as_ref();
            pool.remove(name);
        }

        Ok(())
    }

    /// Returns the state object of the database.
    pub fn state(&self) -> Result<DatabaseState, Error> {
        let state = self.state.lock()?;
        Ok(state.to_owned())
    }

    /// Persists the state of the database to the state file.
    ///
    /// This method requires a Mutex lock to be available.
    /// If the lock is not available, this method will be suspended.
    /// When running this method with other state lock, make sure
    /// to release the lock before calling this method.
    pub fn persist_state(&self) -> Result<(), Error> {
        file::write_binary_file(self.state_file(), &self.state()?)
    }
}

// Write internal database methods here.
impl Database {
    /// Returns the file path where the state is stored.
    fn state_file(&self) -> PathBuf {
        self.root.join("odbstate")
    }

    /// Returns the directory where the indices are stored.
    fn indices_dir(&self) -> PathBuf {
        self.root.join("indices")
    }
}

/// The state of the vector database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseState {
    source: DatabaseURL,
    indices: HashMap<IndexName, IndexRef>,
}

impl DatabaseState {
    /// Restores the database state from a file.
    /// - `path`: Path to the state file.
    pub fn restore(path: impl AsRef<Path>) -> Result<DatabaseState, Error> {
        file::read_binary_file(path)
    }

    /// Updates the source URL of the database state.
    /// - `source`: New source URL.
    pub fn with_source(
        &mut self,
        source: impl Into<DatabaseURL>,
    ) -> Result<(), Error> {
        let source = source.into();
        Self::validate_source(&source)?;
        self.source = source;
        Ok(())
    }

    /// Connects to the source SQL database asynchronously.
    pub async fn async_connect(&self) -> Result<SourceConnection, Error> {
        install_default_drivers();
        Ok(SourceConnection::connect(&self.source).await?)
    }

    /// Connects to the source SQL database.
    pub fn connect(&self) -> Result<SourceConnection, Error> {
        executor::block_on(self.async_connect())
    }

    /// Disconnects from the source SQL database asynchronously.
    /// - `conn`: Database connection.
    pub async fn async_disconnect(conn: SourceConnection) -> Result<(), Error> {
        Ok(conn.close().await?)
    }

    /// Disconnects from the source SQL database.
    /// - `conn`: Database connection.
    pub fn disconnect(conn: SourceConnection) -> Result<(), Error> {
        executor::block_on(Self::async_disconnect(conn))
    }

    /// Validates the connection to the source database.
    ///
    /// This method will try to connect to the source database and
    /// disconnect immediately to validate the connection. If this method
    /// is unable to connect, it will return an error.
    pub fn validate_connection(&self) -> Result<(), Error> {
        let conn = self.connect()?;
        DatabaseState::disconnect(conn)
    }

    /// Returns the type of the source database:
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
    ///
    /// The source URL scheme must be one of:
    /// - sqlite
    /// - mysql
    /// - postgresql
    pub fn validate_source(url: impl Into<DatabaseURL>) -> Result<(), Error> {
        let url = url.into();
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
    config: SourceConfig,
    algorithm: IndexAlgorithm,
    file: IndexFile,
}

impl IndexRef {
    /// Returns the source configuration of the index.
    pub fn config(&self) -> &SourceConfig {
        &self.config
    }

    /// Returns the type of the indexing algorithm of the index.
    pub fn algorithm(&self) -> &IndexAlgorithm {
        &self.algorithm
    }

    /// Returns the file path where the index is stored.
    pub fn file(&self) -> &IndexFile {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::RecordID;
    use sqlx::{Executor, Row};
    use std::sync::MutexGuard;

    const TABLE: &str = "embeddings";
    const TEST_INDEX: &str = "test_index";

    #[test]
    fn test_database_open() {
        assert!(create_test_database().is_ok());
    }

    #[test]
    fn test_database_create_index() -> Result<(), Error> {
        let db = create_test_database()?;

        let index: Index = db.try_get_index(TEST_INDEX)?;
        let index = index.lock()?;
        let metadata = index.metadata();

        assert_eq!(index.len(), 100);
        assert_eq!(metadata.last_inserted, Some(RecordID(100)));
        Ok(())
    }

    #[test]
    fn test_database_refresh_index() -> Result<(), Error> {
        let db = create_test_database()?;
        let query = generate_insert_query(100, 10);
        executor::block_on(db.async_execute_sql(query))?;

        db.refresh_index(TEST_INDEX).unwrap();

        let index: Index = db.try_get_index(TEST_INDEX)?;
        let index = index.lock()?;
        let metadata = index.metadata();

        assert_eq!(index.len(), 110);
        assert_eq!(metadata.last_inserted, Some(RecordID(110)));
        Ok(())
    }

    #[test]
    fn test_database_search_index_basic() {
        let db = create_test_database().unwrap();
        let results = db
            .search_index(TEST_INDEX, vec![0.0; 128], 5, Filters::NONE)
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].id, RecordID(1));
        assert_eq!(results[0].distance, 0.0);
    }

    #[test]
    fn test_database_search_index_advanced() {
        let db = create_test_database().unwrap();
        let results = db
            .search_index(TEST_INDEX, vec![0.0; 128], 5, "data >= 1050")
            .unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].id, RecordID(51));
    }

    #[test]
    fn test_database_delete_index() {
        let db = create_test_database().unwrap();
        db.delete_index(TEST_INDEX).unwrap();

        let state = db.state().unwrap();
        assert!(!state.indices.contains_key(TEST_INDEX));
    }

    #[test]
    fn test_database_indices_pool() -> Result<(), Error> {
        let db = create_test_database()?;

        {
            db.release_indices(vec![TEST_INDEX])?;
            let pool = db.pool()?;
            assert!(!pool.contains_key(TEST_INDEX));
        }

        {
            db.load_indices(vec![TEST_INDEX])?;
            let pool = db.pool()?;
            assert!(pool.contains_key(TEST_INDEX));
        }

        Ok(())
    }

    fn create_test_database() -> Result<Database, Error> {
        let path = PathBuf::from("odb_test");
        if path.try_exists()? {
            fs::remove_dir_all(&path)?;
        }

        let db_path = file::get_tmp_dir()?.join("sqlite.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        let mut db = Database::open(path, Some(db_url.to_owned()))?;
        let state = db.state()?;
        assert_eq!(state.source_type(), SourceType::SQLITE);

        executor::block_on(setup_test_source(&db_url))?;
        create_test_index(&mut db)?;
        Ok(db)
    }

    fn create_test_index(db: &mut Database) -> Result<(), Error> {
        let algorithm = IndexAlgorithm::Flat(ParamsFlat::default());
        let config = SourceConfig::new(TABLE, "id", "vector")
            .with_metadata(vec!["data"]);

        db.create_index(TEST_INDEX, algorithm, config)?;

        let index_ref = db.get_index_ref(TEST_INDEX).unwrap();
        assert_eq!(index_ref.algorithm().name(), "FLAT");
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

    async fn setup_test_source(
        url: impl Into<DatabaseURL>,
    ) -> Result<(), Error> {
        let url = url.into();
        let mut conn = SourceConnection::connect(&url).await?;

        let create_table = format!(
            "CREATE TABLE IF NOT EXISTS {TABLE} (
                id INTEGER PRIMARY KEY,
                vector JSON NOT NULL,
                data INTEGER NOT NULL
            )"
        );

        let insert_records = generate_insert_query(0, 100);
        let drop_table = format!("DROP TABLE IF EXISTS {TABLE}");

        conn.execute(drop_table.as_str()).await?;
        conn.execute(create_table.as_str()).await?;
        conn.execute(insert_records.as_str()).await?;

        let count = {
            let query = format!("SELECT COUNT(*) FROM {TABLE}");
            conn.fetch_one(query.as_str()).await?.get::<i64, usize>(0)
        };

        assert_eq!(count, 100);
        Ok(())
    }

    impl Database {
        fn pool(&self) -> Result<MutexGuard<HashMap<IndexName, Index>>, Error> {
            Ok(self.pool.lock()?)
        }

        async fn async_execute_sql(
            &self,
            query: impl AsRef<str>,
        ) -> Result<(), Error> {
            let mut conn = self.state()?.async_connect().await?;
            conn.execute(query.as_ref()).await?;
            Ok(())
        }
    }
}
