use super::*;
use regex::Regex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseState {
    pub collection_refs: HashMap<String, PathBuf>,
}

struct Directories {
    pub root: PathBuf,
    pub collections_dir: PathBuf,
    pub state_file: PathBuf,
}

impl Directories {
    fn new(root: PathBuf) -> Self {
        let collections_dir = root.join("collections");
        let state_file = root.join("dbstate");
        Self { root, collections_dir, state_file }
    }
}

pub struct Database {
    dirs: Directories,
    state: Lock<DatabaseState>,
}

impl Database {
    pub fn open(dir: PathBuf) -> Result<Self, Error> {
        let dirs = Directories::new(dir);

        let state_file = &dirs.state_file;
        let state = if !state_file.try_exists()? {
            // Creating a collection directory will create the root directory.
            fs::create_dir_all(&dirs.collections_dir)?;
            Self::initialize_state(&state_file)?
        } else {
            Self::read_state(&state_file)?
        };

        let state = Lock::new(state);
        let db = Self { dirs, state };
        Ok(db)
    }
}

// This implementation block contains methods used by the gRPC server.
// We do this to make it easier to test the database logic.
impl Database {
    pub fn _create_collection(&self, name: &str) -> Result<(), Error> {
        Self::validate_collection_name(name)?;

        // Check if the collection already exists.
        let mut state = self.state.write()?;
        if state.collection_refs.contains_key(name) {
            let code = ErrorCode::ClientError;
            let message = format!("Collection already exists: {name}");
            return Err(Error::new(&code, &message));
        }

        // Create the collection directory.
        let uuid = Uuid::new_v4().to_string();
        let collection_dir = self.dirs.collections_dir.join(uuid);

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

    pub fn _delete_collection(&self, name: &str) -> Result<(), Error> {
        let mut state = self.state.write()?;
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
        let state = self.state.read()?;
        let dir = match state.collection_refs.get(collection_name) {
            Some(dir) => dir,
            None => {
                let code = ErrorCode::ClientError;
                let message = format!("No collection name: {collection_name}");
                return Err(Error::new(&code, &message));
            }
        };

        let collection = Collection::open(dir.to_path_buf())?;
        collection.add_fields(fields)?;
        Ok(())
    }

    fn validate_collection_name(name: &str) -> Result<(), Error> {
        if name.is_empty() {
            let code = ErrorCode::ClientError;
            let message = "Collection name cannot be empty";
            return Err(Error::new(&code, message));
        }

        let re = Regex::new(r"^[a-z_]+$").unwrap();
        if !re.is_match(name) {
            let code = ErrorCode::ClientError;
            let message = "Collection name must be lowercase letters \
                with underscores.";
            return Err(Error::new(&code, message));
        }

        Ok(())
    }
}

impl StateMachine<DatabaseState> for Database {
    fn initialize_state(path: &PathBuf) -> Result<DatabaseState, Error> {
        let state = DatabaseState::default();
        FileOps::default().write_binary_file(path, &state)?;
        Ok(state)
    }

    fn read_state(path: &PathBuf) -> Result<DatabaseState, Error> {
        FileOps::default().read_binary_file(path)
    }

    fn state(&self) -> Result<DatabaseState, Error> {
        Ok(self.state.read()?.clone())
    }

    fn persist_state(&self) -> Result<(), Error> {
        let state = self.state.read()?.clone();
        FileOps::default().write_binary_file(&self.dirs.state_file, &state)
    }
}
