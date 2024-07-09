use super::*;
use sqlx::any::install_default_drivers;
use sqlx::{AnyConnection as SourceConnection, Connection};
use tokio::runtime::Runtime;
use url::Url;

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

    /// Creates a new index in the database.
    /// - `name`: Name of the index.
    /// - `config`: Index data source configuration.
    pub fn create_index(
        &mut self,
        name: impl Into<String>,
        config: SourceConfig,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    /// Returns the state object of the database.
    pub fn state(&self) -> &DatabaseState {
        &self.state
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
        Runtime::new()?.block_on(self.async_connect())
    }

    /// Returns the type of the source database.
    /// - sqlite
    /// - mysql
    /// - postgresql
    pub fn source_type(&self) -> String {
        // We can safely unwrap here because
        // we have already validated the source URL.
        let url = self.source.parse::<Url>().unwrap();
        url.scheme().to_owned()
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
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexRef {
    algorithm: IndexAlgorithm,
    file: IndexFile,
}
