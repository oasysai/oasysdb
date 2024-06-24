use super::*;
use crate::proto::database_server::Database as ProtoDatabase;
use crate::proto::CreateCollectionRequest;
use serde::de::DeserializeOwned;

// Database sub-directory structure.
const COLLECTIONS_DIR: &str = "collections";
const INDICES_DIR: &str = "indices";
const TMP_DIR: &str = "tmp";
const SUBDIRS: [&str; 3] = [COLLECTIONS_DIR, INDICES_DIR, TMP_DIR];

// This is where the serialized database states are stored.
const STATE_FILE: &str = "dbstate";

// Type aliases for improved readability.
type CollectionName = String;
type CollectionPath = PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseState {
    pub collection_refs: HashMap<CollectionName, CollectionPath>,
}

impl Default for DatabaseState {
    fn default() -> Self {
        Self { collection_refs: HashMap::new() }
    }
}

pub struct Database {
    directory: PathBuf,
    state: Lock<DatabaseState>,
}

impl Database {
    pub fn open(directory: PathBuf) -> Result<Self, Error> {
        if !directory.try_exists()? {
            Self::initialize_directory(&directory)?;
        }

        let state = Lock::new(DatabaseState::default());
        let mut db = Self { directory, state };

        db.restore_state()?;
        Ok(db)
    }

    pub fn persist_state(&self) -> Result<(), Error> {
        let state = self.state.read()?.clone();
        let state_file = self.directory.join(STATE_FILE);
        self.write_binary_file(&state, &state_file)
    }

    pub fn state(&self) -> Result<DatabaseState, Error> {
        Ok(self.state.read()?.clone())
    }

    fn initialize_directory(directory: &PathBuf) -> Result<(), Error> {
        // Create the parent directory of the database.
        fs::create_dir_all(directory)?;

        // Create the subdirectories for the database.
        for subdir in SUBDIRS {
            let subdir_path = directory.join(subdir);
            fs::create_dir(&subdir_path)?;
        }

        Ok(())
    }

    fn restore_state(&mut self) -> Result<(), Error> {
        let state_file = self.directory.join(STATE_FILE);

        // If there are no state file, return early.
        // This is not an error, as the database may be new.
        if !state_file.try_exists()? {
            return Ok(());
        }

        // Restore the database states.
        self.state = Self::read_binary_file(&state_file)?;
        Ok(())
    }

    fn read_binary_file<T: DeserializeOwned>(
        path: &PathBuf,
    ) -> Result<T, Error> {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        bincode::deserialize_from(reader).map_err(Into::into)
    }

    fn write_binary_file<T: Serialize>(
        &self,
        data: &T,
        path: &PathBuf,
    ) -> Result<(), Error> {
        let filename = path.file_name().ok_or_else(|| {
            // This error should never happen unless the user tinkers with it.
            let code = ErrorCode::FileError;
            let message = format!("Invalid file path: {path:?}");
            Error::new(&code, &message)
        })?;

        // Write the data to a temporary file first.
        // If this fails, the original file will not be overwritten.
        let tmp_path = self.directory.join(TMP_DIR).join(filename);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;

        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, data)?;

        // If the serialization is successful, rename the temporary file.
        fs::rename(&tmp_path, path)?;
        Ok(())
    }
}

// This implementation block contains methods used by the gRPC server.
// We do this to make it easier to test the database logic.
impl Database {
    pub fn _create_collection(&self, name: &str) -> Result<(), Error> {
        let mut state = self.state.write()?;

        // Check if the collection already exists.
        if state.collection_refs.contains_key(name) {
            let code = ErrorCode::ClientError;
            let message = format!("Collection already exists: {name}");
            return Err(Error::new(&code, &message));
        }

        // Create the collection directory.
        let collection_dir = self.directory.join(COLLECTIONS_DIR).join(name);
        fs::create_dir(&collection_dir)?;

        // Update the database state.
        *state = {
            let mut _state = state.clone();
            _state.collection_refs.insert(name.to_string(), collection_dir);
            _state
        };

        // Drop the lock to prevent deadlocks since
        // persist_state also requires the lock.
        drop(state);

        self.persist_state()?;
        Ok(())
    }
}

#[tonic::async_trait]
impl ProtoDatabase for Database {
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let name = request.name;
        self._create_collection(&name)?;
        Ok(Response::new(()))
    }
}
