use super::*;
use crate::protos::database_server::Database as DatabaseService;
use std::io::{BufReader, BufWriter};
use tonic::{Request, Response, Status};

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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Parameters {
    pub dimension: usize,
    pub metric: Metric,
    pub density: usize,
}

#[derive(Debug)]
pub struct Database {
    dir: PathBuf,
    params: Parameters,
    index: Index,
    storage: Storage,
}

impl Database {
    pub fn configure(params: &Parameters) {
        let db = Database {
            dir: Self::dir(),
            params: *params,
            index: Index::new(),
            storage: Storage::new(),
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

        self.persist_as_binary(self.dir.join(PARAMS_FILE), &self.params)?;
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
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}
