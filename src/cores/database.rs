use super::*;
use crate::protos::database_server::Database as DatabaseService;
use std::io::BufWriter;
use tonic::{Request, Response, Status};

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
}

impl Database {
    pub fn configure(params: &Parameters) {
        let dir = Self::dir();
        let db = Database { dir: dir.clone(), params: *params };

        if db.params_file().exists() {
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

            fs::remove_dir_all(&dir).expect("Failed to reset the database");
            println!("The database has been reset successfully");
        }

        db.setup_dir().expect("Failed to setup database directory");
        db.persist_as_binary(db.params_file(), db.params)
            .expect("Failed to persist the parameters");
    }

    pub fn open() -> Self {
        unimplemented!()
    }

    fn dir() -> PathBuf {
        match env::var("ODB_DIR") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => PathBuf::from("oasysdb"),
        }
    }

    fn tmp_dir(&self) -> PathBuf {
        self.dir.join("tmp")
    }

    fn setup_dir(&self) -> Result<(), Box<dyn Error>> {
        if !self.dir.try_exists()? {
            fs::create_dir_all(&self.dir)?;
            fs::create_dir_all(self.dir.join("tmp"))?;
        }

        Ok(())
    }

    fn params_file(&self) -> PathBuf {
        self.dir.join("odb_params")
    }

    fn persist_as_binary<T: Serialize>(
        &self,
        path: impl AsRef<Path>,
        data: T,
    ) -> Result<(), Box<dyn Error>> {
        let file_name = path.as_ref().file_name().unwrap();
        let tmp_file = self.tmp_dir().join(file_name);
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
