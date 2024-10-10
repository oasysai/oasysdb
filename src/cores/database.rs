use super::*;
use protos::database_server::Database as DatabaseService;
use std::io::{BufReader, BufWriter};
use tonic::{Request, Response};

const TMP_DIR: &str = "tmp";
const PARAMS_FILE: &str = "odb_params";
const STORAGE_FILE: &str = "odb_storage";
const INDEX_FILE: &str = "odb_index";

/// Database parameters.
///
/// Fields:
/// - dimension: Vector dimension.
/// - metric: Metric to calculate distance.
/// - density: Max number of records per IVF cluster.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Parameters {
    pub dimension: usize,
    pub metric: Metric,
    pub density: usize,
}

/// Dynamic query-time parameters.
///
/// Fields:
/// - probes: Suggested number of clusters to visit.
/// - radius: Maximum distance to include in the result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QueryParameters {
    pub probes: usize,
    pub radius: f32,
}

impl Default for QueryParameters {
    /// Default query parameters:
    /// - probes: 32
    /// - radius: ∞
    fn default() -> Self {
        QueryParameters { probes: 32, radius: f32::INFINITY }
    }
}

impl TryFrom<protos::QueryParameters> for QueryParameters {
    type Error = Status;
    fn try_from(value: protos::QueryParameters) -> Result<Self, Self::Error> {
        Ok(QueryParameters {
            probes: value.probes as usize,
            radius: value.radius,
        })
    }
}

/// Database snapshot statistics.
///
/// The snapshot statistics include the information that might be useful
/// for monitoring the state of the database. This stats will be returned
/// by the `create_snapshot` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SnapshotStats {
    pub count: usize,
}

impl From<SnapshotStats> for protos::SnapshotResponse {
    fn from(value: SnapshotStats) -> Self {
        protos::SnapshotResponse { count: value.count as i32 }
    }
}

#[derive(Debug)]
pub struct Database {
    dir: PathBuf,
    params: Parameters,
    index: RwLock<Index>,
    storage: RwLock<Storage>,
}

impl Database {
    pub fn configure(params: &Parameters) {
        let index = Index::new()
            .with_metric(params.metric)
            .with_density(params.density);

        let db = Database {
            dir: Self::dir(),
            params: *params,
            index: RwLock::new(index),
            storage: RwLock::new(Storage::new()),
        };

        if db.dir.join(PARAMS_FILE).exists() {
            let stdin = std::io::stdin();
            let overwrite = {
                eprint!("Database is already configured. Overwrite? (y/n): ");
                let mut input = String::new();
                stdin.read_line(&mut input).unwrap();
                matches!(input.to_lowercase().trim(), "y")
            };

            if !overwrite {
                return;
            }

            fs::remove_dir_all(&db.dir).expect("Failed to reset the database");
            println!("The database has been reset successfully");
        }

        db.setup_dir().expect("Failed to setup database directory");
    }

    pub fn open() -> Result<Self, Box<dyn Error>> {
        let dir = Self::dir();
        let params = Self::load_binary(dir.join(PARAMS_FILE))?;
        let index = Self::load_binary(dir.join(INDEX_FILE))?;
        let storage: Storage = Self::load_binary(dir.join(STORAGE_FILE))?;

        let count = storage.count();
        tracing::info!("Restored {count} record(s) from the disk");

        Ok(Database {
            dir,
            params,
            index: RwLock::new(index),
            storage: RwLock::new(storage),
        })
    }

    fn dir() -> PathBuf {
        match env::var("ODB_DIR") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => PathBuf::from("oasysdb"),
        }
    }

    fn setup_dir(&self) -> Result<(), Box<dyn Error>> {
        if self.dir.try_exists()? {
            return Ok(());
        }

        fs::create_dir_all(&self.dir)?;
        fs::create_dir_all(self.dir.join(TMP_DIR))?;

        self.create_snapshot()?;
        Ok(())
    }

    fn load_binary<T: DeserializeOwned>(
        path: impl AsRef<Path>,
    ) -> Result<T, Box<dyn Error>> {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        Ok(bincode::deserialize_from(reader)?)
    }

    fn persist_as_binary<T: Serialize>(
        &self,
        path: impl AsRef<Path>,
        data: T,
    ) -> Result<(), Box<dyn Error>> {
        let file_name = path.as_ref().file_name().unwrap();
        let tmp_file = self.dir.join(TMP_DIR).join(file_name);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_file)?;

        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, &data)?;
        fs::rename(&tmp_file, &path)?;
        Ok(())
    }

    pub fn create_snapshot(&self) -> Result<SnapshotStats, Box<dyn Error>> {
        self.persist_as_binary(self.dir.join(PARAMS_FILE), self.params)?;

        let index = self.index.read().unwrap();
        self.persist_as_binary(self.dir.join(INDEX_FILE), &*index)?;

        let storage = self.storage.read().unwrap();
        self.persist_as_binary(self.dir.join(STORAGE_FILE), &*storage)?;

        let count = storage.count();
        tracing::info!("Created a snapshot with {count} record(s)");

        Ok(SnapshotStats { count })
    }

    fn validate_dimension(&self, vector: &Vector) -> Result<(), Status> {
        if vector.len() != self.params.dimension {
            return Err(Status::invalid_argument(format!(
                "Invalid vector dimension: expected {}, got {}",
                self.params.dimension,
                vector.len()
            )));
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl DatabaseService for Arc<Database> {
    async fn heartbeat(
        &self,
        _request: Request<()>,
    ) -> Result<Response<protos::HeartbeatResponse>, Status> {
        let response = protos::HeartbeatResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        Ok(Response::new(response))
    }

    async fn snapshot(
        &self,
        _request: Request<()>,
    ) -> Result<Response<protos::SnapshotResponse>, Status> {
        let stats = self.create_snapshot().map_err(|e| {
            let message = format!("Failed to create a snapshot: {e}");
            Status::internal(message)
        })?;

        Ok(Response::new(stats.into()))
    }

    async fn insert(
        &self,
        request: Request<protos::InsertRequest>,
    ) -> Result<Response<protos::InsertResponse>, Status> {
        let record = match request.into_inner().record {
            Some(record) => Record::try_from(record)?,
            None => {
                let message = "Record data is required for insertion";
                return Err(Status::invalid_argument(message));
            }
        };

        self.validate_dimension(&record.vector)?;

        let id = RecordID::new();

        // Insert the record into the storage.
        // This operation must be done before updating the index. Otherwise,
        // the index won't have access to the record data.
        let mut storage = self.storage.write().unwrap();
        storage.insert(&id, &record)?;

        let mut index = self.index.write().unwrap();
        index.insert(&id, &record, storage.records())?;

        tracing::info!("Inserted a new record with ID: {id}");
        Ok(Response::new(protos::InsertResponse { id: id.to_string() }))
    }

    async fn get(
        &self,
        request: Request<protos::GetRequest>,
    ) -> Result<Response<protos::GetResponse>, Status> {
        let request = request.into_inner();
        let id = request.id.parse::<RecordID>()?;

        let storage = self.storage.read().unwrap();
        let record = storage.get(&id)?.to_owned();

        let response = protos::GetResponse { record: Some(record.into()) };
        Ok(Response::new(response))
    }

    async fn delete(
        &self,
        request: Request<protos::DeleteRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let id = request.id.parse::<RecordID>()?;

        let mut index = self.index.write().unwrap();
        index.delete(&id)?;

        let mut storage = self.storage.write().unwrap();
        storage.delete(&id)?;

        tracing::info!("Deleted a record with ID: {id}");
        Ok(Response::new(()))
    }

    async fn update(
        &self,
        request: Request<protos::UpdateRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let id = request.id.parse::<RecordID>()?;

        let mut metadata = HashMap::new();
        for (key, value) in request.metadata {
            metadata.insert(key, value.try_into()?);
        }

        let mut storage = self.storage.write().unwrap();
        storage.update(&id, &metadata)?;

        tracing::info!("Updated metadata for a record: {id}");
        Ok(Response::new(()))
    }

    async fn query(
        &self,
        request: Request<protos::QueryRequest>,
    ) -> Result<Response<protos::QueryResponse>, Status> {
        let request = request.into_inner();
        let vector = match request.vector {
            Some(vector) => Vector::try_from(vector)?,
            None => {
                let message = "Vector is required for query operation";
                return Err(Status::invalid_argument(message));
            }
        };

        self.validate_dimension(&vector)?;

        let k = request.k as usize;
        if k == 0 {
            let message = "Invalid k value, k must be greater than 0";
            return Err(Status::invalid_argument(message));
        }

        let filter = Filters::try_from(request.filter.as_str())?;

        let params = match request.params {
            Some(params) => QueryParameters::try_from(params)?,
            None => QueryParameters::default(),
        };

        let storage = self.storage.read().unwrap();
        let records = storage.records();

        let index = self.index.read().unwrap();
        let results = index
            .query(&vector, k, &filter, &params, records)?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(Response::new(protos::QueryResponse { results }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_open() {
        let db = setup_db();
        assert_eq!(db.params, Parameters::default());
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let db = setup_db();
        let request = Request::new(());
        let response = db.heartbeat(request).await.unwrap();
        assert_eq!(response.get_ref().version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_insert() {
        let params = Parameters::default();
        let db = setup_db();

        let vector = Vector::random(params.dimension);
        let request = Request::new(protos::InsertRequest {
            record: Some(protos::Record {
                vector: Some(vector.into()),
                metadata: std::collections::HashMap::new(),
            }),
        });

        let response = db.insert(request).await.unwrap();
        assert!(response.get_ref().id.parse::<Uuid>().is_ok());
        assert_eq!(db.storage.read().unwrap().records().len(), 1);
    }

    fn setup_db() -> Arc<Database> {
        if Database::dir().exists() {
            fs::remove_dir_all(Database::dir()).unwrap();
        }

        let params = Parameters::default();
        Database::configure(&params);
        Arc::new(Database::open().unwrap())
    }

    impl Default for Parameters {
        fn default() -> Self {
            Parameters {
                dimension: 128,
                metric: Metric::Euclidean,
                density: 64,
            }
        }
    }
}
