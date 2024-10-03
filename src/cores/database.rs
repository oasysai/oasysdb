use super::*;
use crate::protos;
use crate::protos::database_server::Database as DatabaseService;
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
        let storage = Self::load_binary(dir.join(STORAGE_FILE))?;

        let db = Database { dir, params, index, storage };
        Ok(db)
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
        fs::create_dir_all(self.dir.join("tmp"))?;

        self.persist_as_binary(self.dir.join(PARAMS_FILE), self.params)?;
        self.persist_as_binary(self.dir.join(INDEX_FILE), &self.index)?;
        self.persist_as_binary(self.dir.join(STORAGE_FILE), &self.storage)?;

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

    async fn insert(
        &self,
        request: Request<protos::InsertRequest>,
    ) -> Result<Response<protos::InsertResponse>, Status> {
        let record = match request.into_inner().record {
            Some(record) => Record::try_from(record)?,
            None => return Err(Status::invalid_argument("Record is required")),
        };

        if record.vector.len() != self.params.dimension {
            return Err(Status::invalid_argument(format!(
                "Invalid vector dimension: expected {}, got {}",
                self.params.dimension,
                record.vector.len()
            )));
        }

        let id = RecordID::new();

        // Insert the record into the storage.
        // This operation must be done before updating the index. Otherwise,
        // the index won't have access to the record data.
        let mut storage = self.storage.write().unwrap();
        storage.insert(&id, &record)?;

        let mut index = self.index.write().unwrap();
        index.insert(&id, &record, storage.records())?;

        Ok(Response::new(protos::InsertResponse { id: id.to_string() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
