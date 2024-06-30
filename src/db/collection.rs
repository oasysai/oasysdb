use super::*;
use array::downcast_array;
use arrow::compute::concat_batches;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionState {
    pub batch_size: usize,
    pub count: usize,
    pub dimension: usize,
    pub schema: Schema,
    pub dir: Directory,
    /// Tracker of the next internal ID to assign to a record.
    next_id: u32,
}

impl CollectionState {
    fn new(dir: PathBuf) -> Result<Self, Error> {
        let field_id = Field::new("internal_id", DataType::Int32, false);

        let vector_type = MetadataType::Vector.into();
        let field_vector = Field::new("vector", vector_type, false);

        let mut state = Self {
            schema: Schema::new(vec![field_id, field_vector]),
            dir: Directory::new(dir),
            batch_size: 1000,
            count: 0,
            dimension: 0,
            next_id: 1,
        };

        state.create_data_file()?;
        Ok(state)
    }

    fn create_data_file(&mut self) -> Result<PathBuf, Error> {
        // The filename would be something like: cdata0000001.
        let index = self.dir.data_files.len() + 1;
        let filename = format!("cdata{index:0>7}");
        let data_file = self.dir.root.join(filename);

        let schema_ref = Arc::new(self.schema.clone());

        // Create a new data file with an empty record batch.

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&data_file)?;

        let writer = BufWriter::new(file);
        let mut file_writer = FileWriter::try_new(writer, &schema_ref)?;

        let record = RecordBatch::new_empty(schema_ref);
        file_writer.write(&record)?;
        file_writer.finish()?;

        self.dir.data_files.push(data_file.clone());
        Ok(data_file)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub root: PathBuf,
    pub state_file: PathBuf,
    pub data_files: Vec<PathBuf>,
}

impl Directory {
    fn new(root: PathBuf) -> Self {
        let state_file = root.join("cstate");
        Self { root, state_file, data_files: vec![] }
    }
}

pub struct Collection {
    state: Lock<CollectionState>,
}

impl Collection {
    pub fn open(dir: PathBuf) -> Result<Self, Error> {
        if !dir.try_exists()? {
            fs::create_dir_all(&dir)?;
        }

        let state_file = dir.join("cstate");
        let state = if !state_file.try_exists()? {
            Self::initialize_state(&dir)?
        } else {
            Self::read_state(&state_file)?
        };

        let state = Lock::new(state);
        let collection = Self { state };
        Ok(collection)
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

    pub fn insert_records(
        &self,
        field_names: &[String],
        records: &[Arc<dyn Array>],
    ) -> Result<(), Error> {
        let mut state = self.state.write()?;

        let mut record_map: HashMap<String, Arc<dyn Array>> = field_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), records[i].clone()))
            .collect();

        // It's safe to unwrap here because the vector field has been checked in
        // the database service before calling this method.
        let vector_array = record_map.get("vector").unwrap();

        let data_size = vector_array.len();
        let dimension = {
            let array: ListArray = downcast_array(vector_array.as_ref());
            let vector: Float32Array = downcast_array(array.value(0).as_ref());
            vector.len()
        };

        if dimension == 0 {
            let code = ErrorCode::ClientError;
            let message = "Vector cannot be empty";
            return Err(Error::new(&code, message));
        }

        // If it's the first record, we need to update the dimension.
        if state.count == 0 && state.dimension == 0 {
            state.dimension = dimension;
        }

        // Ensure all vectors have the same dimension.
        self.validate_vectors(vector_array, dimension)?;

        let schema = state.schema.clone();
        let fields = schema.all_fields();

        // Create a column array for internal_id.
        let internal_id: Vec<Option<i32>> = (state.next_id..)
            .take(data_size)
            .map(|id| Some(id as i32))
            .collect();
        let internal_id_array = Arc::new(Int32Array::from(internal_id));

        record_map.insert("internal_id".to_string(), internal_id_array);

        // Check for missing fields in the record and create a
        // column array for each missing field with null values.
        // This is necessary to ensure that all fields are present.
        let create_missing_array = |field: &Field| {
            let data_type = field.data_type().clone().into();
            let array = match data_type {
                MetadataType::Integer => Int32Array::null_array(data_size),
                MetadataType::Float => Float32Array::null_array(data_size),
                MetadataType::String => StringArray::null_array(data_size),
                MetadataType::Boolean => BooleanArray::null_array(data_size),
                MetadataType::Vector => ListArray::null_array(data_size),
            };

            (field.name().to_string(), array as Arc<dyn Array>)
        };

        let missing_fields: HashMap<String, Arc<dyn Array>> = fields
            .into_iter()
            .filter(|field| !record_map.contains_key(field.name()))
            .map(create_missing_array)
            .collect();

        // Merge the missing fields with the record map.
        record_map.extend(missing_fields);

        // Convert the record map to columns in order based on the schema.
        let extract_array = |field: &Arc<Field>| {
            let name = field.name();
            let array = record_map.get(name).unwrap();
            array.clone()
        };

        let columns = schema.fields.iter().map(extract_array).collect();

        // Create a record batch from the record map.
        let schemaref = Arc::new(schema.clone());
        let record_batch = RecordBatch::try_new(schemaref.clone(), columns)?;

        // OasysDB limits the number of record batches in a data file to 1.
        // Per record batch, there can be a maximum of 1000 records by default.

        // The behavior is as follows:
        // 1. If the last data file is empty, write the record batch to it.
        // 2. If the last data file is not empty, combine the last record batch
        //    with the new record batch and write the combined record batch to
        //    the last data file until it reaches the batch size.

        let data_files = &mut state.dir.data_files;
        let file_ops = FileOps::default();

        // Also, we can unwrap here because the data files won't be None.
        let last_data_file = data_files.last().unwrap();
        let last_record_batch = file_ops.read_ipc_file(last_data_file)?;

        let record_batch = if last_record_batch.num_rows() != 0 {
            let batches = vec![&last_record_batch, &record_batch];
            concat_batches(&schemaref, batches)?
        } else {
            record_batch
        };

        let mut files_to_write = vec![last_data_file.clone()];

        // This determines the number of new files to create.
        // Let's say the batch size is 1000 and the combined record batch
        // has 1500 records. This means we need to create 1 new file because
        // the first 1000 records will be written to the last data file and
        // the remaining 500 records will be written to the new file.
        let num_new_file = {
            let size = record_batch.num_rows();
            let remain = size.saturating_sub(state.batch_size) as f32;
            let div = remain / state.batch_size as f32;
            div.ceil() as usize
        };

        for _ in 0..num_new_file {
            let data_file = state.create_data_file()?;
            files_to_write.push(data_file);
        }

        FileOps::default().write_ipc_files(
            &files_to_write,
            &record_batch,
            state.batch_size,
        )?;

        // Update and persist the state.
        state.count += data_size;
        state.next_id += data_size as u32;
        *state = state.clone();

        // Drop the state lock before persisting the state.
        // This prevents deadlocks since persist_state also requires the lock.
        drop(state);
        self.persist_state()?;

        Ok(())
    }
}

impl StateMachine<CollectionState> for Collection {
    fn initialize_state(
        root: impl Into<PathBuf>,
    ) -> Result<CollectionState, Error> {
        let state = CollectionState::new(root.into())?;
        FileOps::default().write_binary_file(&state.dir.state_file, &state)?;
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
        file_ops.write_binary_file(&state.dir.state_file, &state)
    }
}
