use super::*;
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
            Self::initialize_state(state_file)?
        } else {
            Self::read_state(state_file)?
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
        let dir = self.get_collection_dir(collection_name)?;
        let collection = Collection::open(dir)?;
        collection.add_fields(fields)?;
        Ok(())
    }

    pub fn _remove_fields(
        &self,
        collection_name: &str,
        field_names: &[String],
    ) -> Result<(), Error> {
        let dir = self.get_collection_dir(collection_name)?;
        let collection = Collection::open(dir)?;
        collection.remove_fields(field_names)?;
        Ok(())
    }

    fn get_collection_dir(&self, name: &str) -> Result<PathBuf, Error> {
        let state = self.state.read()?;
        match state.collection_refs.get(name) {
            Some(dir) => Ok(dir.clone()),
            None => {
                let code = ErrorCode::ClientError;
                let message = format!("No collection name: {name}");
                Err(Error::new(&code, &message))
            }
        }
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
        FileOps::default().write_binary_file(&self.dirs.state_file, &state)
    }
}
