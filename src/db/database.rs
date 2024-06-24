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
const STATE_FILE: &str = "state";

// Type aliases for improved readability.
type CollectionName = String;
type CollectionPath = PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseState {
    collection_refs: HashMap<CollectionName, CollectionPath>,
}

impl Default for DatabaseState {
    fn default() -> Self {
        Self { collection_refs: HashMap::new() }
    }
}

pub struct Database {
    directory: PathBuf,
    state: DatabaseState,
}

impl Database {
    pub fn open(directory: PathBuf) -> Result<Self, Error> {
        if !directory.try_exists()? {
            Self::initialize_directory(&directory)?;
        }

        let state = DatabaseState::default();
        let mut db = Self { directory, state };

        db.restore_states()?;
        Ok(db)
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

    fn restore_states(&mut self) -> Result<(), Error> {
        let state_file = self.directory.join(STATE_FILE);

        // If there are no state file, return early.
        // This is not an error, as the database may be new.
        if !state_file.try_exists()? {
            return Ok(());
        }

        // Restore the database states.
        self.state = Self::deserialize_binary_file(&state_file)?;
        Ok(())
    }

    fn deserialize_binary_file<T: DeserializeOwned>(
        path: &PathBuf,
    ) -> Result<T, Error> {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        bincode::deserialize_from(reader).map_err(Into::into)
    }
}

#[tonic::async_trait]
impl ProtoDatabase for Database {
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        unimplemented!();
    }
}
