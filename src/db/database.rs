use super::*;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseState {
    pub collection_refs: HashMap<String, PathBuf>,
}

struct Directory {
    pub collections_dir: PathBuf,
    pub state_file: PathBuf,
}

impl Directory {
    fn new(root: PathBuf) -> Self {
        let collections_dir = root.join("collections");
        let state_file = root.join("dbstate");
        Self { collections_dir, state_file }
    }
}

pub struct Database {
    dir: Directory,
    state: Lock<DatabaseState>,
}

impl Database {
    pub fn open(dir: PathBuf) -> Result<Self, Error> {
        let dir = Directory::new(dir);

        let state_file = &dir.state_file;
        let state = if !state_file.try_exists()? {
            // Creating a collection directory will create the root directory.
            fs::create_dir_all(&dir.collections_dir)?;
            Self::initialize_state(state_file)?
        } else {
            Self::read_state(state_file)?
        };

        let state = Lock::new(state);
        let db = Self { dir, state };
        Ok(db)
    }
}

// This implementation block contains methods used by the gRPC server.
// We do this to make it easier to test the database logic.
impl Database {
    pub fn _create_collection(&self, name: &str) -> Result<(), Error> {
        Collection::validate_name(name)?;

        // Check if the collection already exists.
        let mut state = self.state.write()?;
        if state.collection_refs.contains_key(name) {
            let code = ErrorCode::ClientError;
            let message = format!("Collection already exists: {name}");
            return Err(Error::new(&code, &message));
        }

        // Create the collection directory.
        let uuid = Uuid::new_v4().to_string();
        let collection_dir = self.dir.collections_dir.join(uuid);

        // Initialize the collection.
        Collection::open(collection_dir.to_path_buf())?;

        // Update the database state.
        state.collection_refs.insert(name.to_string(), collection_dir);
        *state = state.clone();

        // Drop the lock to prevent deadlocks since
        // persist_state also requires the lock.
        drop(state);

        self.persist_state()?;
        Ok(())
    }

    pub fn _get_collection(&self, name: &str) -> Result<Collection, Error> {
        let state = self.state.read()?;

        if name.is_empty() {
            let code = ErrorCode::ClientError;
            let message = "Collection name cannot be empty";
            return Err(Error::new(&code, message));
        }

        // Get the directory where the collection is
        // persisted from the database state.
        let dir = match state.collection_refs.get(name) {
            Some(dir) => dir.clone(),
            None => {
                let code = ErrorCode::NotFoundError;
                let message = format!("Collection not found: {name}");
                return Err(Error::new(&code, &message));
            }
        };

        Collection::open(dir)
    }

    pub fn _delete_collection(&self, name: &str) -> Result<(), Error> {
        let mut state = self.state.write()?;

        // This makes the method idempotent.
        if !state.collection_refs.contains_key(name) {
            return Ok(());
        }

        // Delete the collection directory.
        // We can unwrap here because we checked if the collection exists.
        let collection_dir = state.collection_refs.remove(name).unwrap();
        fs::remove_dir_all(collection_dir)?;

        // Update the database state.
        *state = state.clone();
        drop(state);

        self.persist_state()?;
        Ok(())
    }

    pub fn _add_fields(
        &self,
        collection_name: &str,
        fields: impl Into<Fields>,
    ) -> Result<(), Error> {
        let collection = self._get_collection(collection_name)?;
        collection.add_fields(fields)?;
        Ok(())
    }

    pub fn _remove_fields(
        &self,
        collection_name: &str,
        field_names: &[String],
    ) -> Result<(), Error> {
        let collection = self._get_collection(collection_name)?;
        collection.remove_fields(field_names)?;
        Ok(())
    }

    pub fn _insert_records(
        &self,
        collection_name: &str,
        field_names: &[String],
        records: &[Arc<dyn Array>],
    ) -> Result<(), Error> {
        let collection = self._get_collection(collection_name)?;
        collection.insert_records(field_names, records)?;
        Ok(())
    }
}

impl StateMachine<DatabaseState> for Database {
    fn initialize_state(
        path: impl Into<PathBuf>,
    ) -> Result<DatabaseState, Error> {
        let state = DatabaseState::default();
        FileOps::default().write_binary_file(&path.into(), &state)?;
        Ok(state)
    }

    fn read_state(path: impl Into<PathBuf>) -> Result<DatabaseState, Error> {
        FileOps::default().read_binary_file(&path.into())
    }

    fn state(&self) -> Result<DatabaseState, Error> {
        Ok(self.state.read()?.clone())
    }

    fn persist_state(&self) -> Result<(), Error> {
        let state = self.state.read()?.clone();
        FileOps::default().write_binary_file(&self.dir.state_file, &state)
    }
}
