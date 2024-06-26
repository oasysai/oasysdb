use super::*;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionState {
    pub schema: Schema,
    pub count: usize,
}

impl CollectionState {
    fn new() -> Self {
        let field_id = Field::new("internal_id", DataType::Int32, false);

        let vector_type = MetadataType::Vector.into();
        let field_vector = Field::new("vector", vector_type, false);

        // The default schema for a new collection contains two fields:
        // internal_id and vector.
        let schema = Schema::new(vec![field_id, field_vector]);
        Self { schema, count: 0 }
    }
}

struct Directories {
    pub root: PathBuf,
    pub state_file: PathBuf,
    pub data_file: PathBuf,
}

impl Directories {
    fn new(root: PathBuf) -> Self {
        let state_file = root.join("cstate");
        let data_file = root.join("cdata");
        Self { root, state_file, data_file }
    }
}

pub struct Collection {
    dirs: Directories,
    state: Lock<CollectionState>,
}

impl Collection {
    pub fn open(dir: PathBuf) -> Result<Self, Error> {
        if !dir.try_exists()? {
            fs::create_dir_all(&dir)?;
        }

        let dirs = Directories::new(dir);
        let state = if !dirs.state_file.try_exists()? {
            let state = Self::initialize_state(&dirs.state_file)?;
            Self::initialize_data_file(&dirs.data_file, &state.schema)?;
            state
        } else {
            Self::read_state(&dirs.state_file)?
        };

        let state = Lock::new(state);
        let collection = Self { dirs, state };
        Ok(collection)
    }

    /// Creates an empty data file for the collection.
    /// This method should only be called once, when the collection is created.
    fn initialize_data_file(
        path: &PathBuf,
        schema: &Schema,
    ) -> Result<(), Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let writer = BufWriter::new(file);
        let mut file_writer = FileWriter::try_new(writer, schema)?;

        let record = RecordBatch::new_empty(Arc::new(schema.clone()));
        file_writer.write(&record)?;

        file_writer.finish()?;
        Ok(())
    }

    pub fn add_fields(&self, fields: impl Into<Fields>) -> Result<(), Error> {
        let mut state = self.state.write()?;

        // OasysDB doesn't support adding fields to a non-empty
        // collection due to the nature of the indexing system.
        if state.count > 0 {
            let code = ErrorCode::ClientError;
            let message = "Unable to add fields to a non-empty collection";
            return Err(Error::new(&code, message));
        }

        // Create a new schema with the new field.
        let schema = &state.schema;
        let schemas = vec![schema.clone(), Schema::new(fields)];
        let new_schema = Schema::try_merge(schemas)?;

        // Update the state and data.
        state.schema = new_schema;
        *state = state.clone();

        drop(state);
        self.persist_state()?;
        Ok(())
    }

    pub fn remove_fields(&self, field_names: &[String]) -> Result<(), Error> {
        let mut state = self.state.write()?;
        let schema = &state.schema;

        // Just like adding fields, removing fields from a non-empty
        // collection is not supported in OasysDB.
        if state.count > 0 {
            let code = ErrorCode::ClientError;
            let message = "Unable to remove fields from a non-empty collection";
            return Err(Error::new(&code, message));
        }

        // OasysDB has 2 default fields which can't be removed:
        // internal_id and vector.
        let default = ["internal_id", "vector"];
        if field_names.iter().any(|name| default.contains(&name.as_str())) {
            let code = ErrorCode::ClientError;
            let message = "Unable to remove default fields";
            return Err(Error::new(&code, message));
        }

        // Check if all the fields to be removed exist in the schema.
        // Abort if any of the fields do not exist.
        if field_names.iter().any(|name| schema.fields.find(name).is_none()) {
            let code = ErrorCode::ClientError;
            let message = "One or more fields do not exist in the schema.";
            return Err(Error::new(&code, message));
        }

        let fields = schema
            .all_fields()
            .into_iter()
            .filter(|field| !field_names.contains(field.name()))
            .cloned()
            .collect::<Vec<_>>();

        // Create a new schema without the specified fields.
        let new_schema = Schema::new(fields);

        // Update the state and data.
        state.schema = new_schema;
        *state = state.clone();

        drop(state);
        self.persist_state()?;
        Ok(())
    }

    pub fn validate_name(name: &str) -> Result<(), Error> {
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

impl StateMachine<CollectionState> for Collection {
    fn initialize_state(
        path: impl Into<PathBuf>,
    ) -> Result<CollectionState, Error> {
        let state = CollectionState::new();
        FileOps::default().write_binary_file(&path.into(), &state)?;
        Ok(state)
    }

    fn read_state(path: impl Into<PathBuf>) -> Result<CollectionState, Error> {
        FileOps::default().read_binary_file(&path.into())
    }

    fn state(&self) -> Result<CollectionState, Error> {
        Ok(self.state.read()?.clone())
    }

    fn persist_state(&self) -> Result<(), Error> {
        let state = self.state.read()?.clone();
        let file_ops = FileOps::default();
        file_ops.write_binary_file(&self.dirs.state_file, &state)
    }
}
