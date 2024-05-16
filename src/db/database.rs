use super::*;

/// The directory where collections are stored in the database.
const COLLECTIONS_DIR: &str = "collections";

/// The database record for the persisted vector collection.
#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionRecord {
    /// Name of the collection.
    pub name: String,
    /// File path where the collection is stored.
    pub path: String,
    /// Number of vector records in the collection.
    pub count: usize,
    /// Timestamp when the collection was created.
    pub created_at: usize,
    /// Timestamp when the collection was last updated.
    pub updated_at: usize,
}

/// The database storing vector collections.
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.database"))]
pub struct Database {
    collections: Db,
    count: usize,
    path: String,
}

/// Python only methods.
#[cfg(feature = "py")]
#[pymethods]
impl Database {
    #[staticmethod]
    #[pyo3(name = "new")]
    fn py_new(path: &str) -> PyResult<Self> {
        Self::new(path).map_err(|e| e.into())
    }

    #[new]
    fn py_open(path: &str) -> PyResult<Self> {
        Self::open(path).map_err(|e| e.into())
    }

    fn __len__(&self) -> usize {
        self.len()
    }
}

// Mixed Rust and Python methods.
#[cfg_attr(feature = "py", pymethods)]
impl Database {
    /// Gets a collection from the database.
    /// * `name`: Name of the collection.
    pub fn get_collection(&self, name: &str) -> Result<Collection, Error> {
        // Retrieve the collection record from the database.
        let record: CollectionRecord = match self.collections.get(name)? {
            Some(value) => bincode::deserialize(&value)?,
            None => return Err(Error::collection_not_found()),
        };

        self.read_from_file(&record.path)
    }

    /// Saves new or update existing collection to the database.
    /// * `name`: Name of the collection.
    /// * `collection`: Vector collection to save.
    pub fn save_collection(
        &mut self,
        name: &str,
        collection: &Collection,
    ) -> Result<(), Error> {
        // This variable is required since some operations require
        // the write_to_file method to succeed.
        let mut new = false;

        let mut record: CollectionRecord;
        let path: String;

        // Check if it's a new collection.
        if !self.collections.contains_key(name)? {
            new = true;
            path = self.create_new_collection_path(name)?;

            // Create a new collection record.
            let timestamp = self.get_timestamp();
            record = CollectionRecord {
                name: name.to_string(),
                path: path.clone(),
                count: collection.len(),
                created_at: timestamp,
                updated_at: timestamp,
            };
        } else {
            let bytes = self.collections.get(name)?.unwrap().to_vec();
            record = bincode::deserialize(&bytes)?;
            path = record.path.clone();

            // Update the record values.
            record.count = collection.len();
            record.updated_at = self.get_timestamp();
        }

        // Write the collection to a file.
        self.write_to_file(&path, collection)?;

        // Insert or update the collection record in the database.
        let bytes = bincode::serialize(&record)?;
        self.collections.insert(name, bytes)?;

        // If it's a new collection, update the count.
        if new {
            self.count += 1;
        }

        Ok(())
    }

    /// Deletes a collection from the database.
    /// * `name`: Collection name to delete.
    pub fn delete_collection(&mut self, name: &str) -> Result<(), Error> {
        let record: CollectionRecord = match self.collections.get(name)? {
            Some(value) => bincode::deserialize(&value)?,
            None => return Err(Error::collection_not_found()),
        };

        // Delete the collection file first before removing
        // the reference from the database.
        self.delete_file(&record.path)?;

        self.collections.remove(name)?;
        self.count -= 1;
        Ok(())
    }

    /// Returns the number of collections in the database.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the database is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Flushes dirty IO buffers and syncs the data to disk.
    /// Returns bytes flushed.
    pub fn flush(&self) -> Result<usize, Error> {
        let bytes = self.collections.flush()?;
        Ok(bytes)
    }

    /// Asynchronously performs flush operation.
    pub async fn async_flush(&self) -> Result<usize, Error> {
        let bytes = self.collections.flush_async().await?;
        Ok(bytes)
    }
}

impl Database {
    /// Re-creates and opens the database at the given path.
    /// This method will delete the database if it exists.
    /// * `path`: Directory to store the database.
    pub fn new(path: &str) -> Result<Self, Error> {
        // Remove the database dir if it exists.
        if Path::new(path).exists() {
            remove_dir_all(path)?;
        }

        // Setup the directory where collections will be stored.
        Self::setup_collections_dir(path)?;

        // Using sled::Config to prevent name collisions
        // with collection's Config.
        let config = sled::Config::new().path(path);
        let collections = config.open()?;
        Ok(Self { collections, count: 0, path: path.to_string() })
    }

    /// Opens existing or creates new database.
    /// If the database doesn't exist, it will be created.
    /// * `path`: Directory to store the database.
    pub fn open(path: &str) -> Result<Self, Error> {
        let collections = sled::open(path)?;
        let count = collections.len();
        Self::setup_collections_dir(path)?;
        Ok(Self { collections, count, path: path.to_string() })
    }

    /// Serializes and writes the collection to a file.
    /// * `path`: File path to write the collection to.
    /// * `collection`: Vector collection to write.
    fn write_to_file(
        &self,
        path: &str,
        collection: &Collection,
    ) -> Result<(), Error> {
        let data = bincode::serialize(collection)?;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let mut writer = BufWriter::new(file);
        writer.write_all(&data)?;
        Ok(())
    }

    /// Reads and deserializes the collection from a file.
    /// * `path`: File path to read the collection from.
    fn read_from_file(&self, path: &str) -> Result<Collection, Error> {
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        // Deserialize the collection.
        let collection = bincode::deserialize(&data)?;
        Ok(collection)
    }

    /// Deletes a file at the given path.
    fn delete_file(&self, path: &str) -> Result<(), Error> {
        remove_file(path)?;
        Ok(())
    }

    /// Returns the path where the collection will be stored.
    /// * `name`: Name of the collection.
    fn create_new_collection_path(&self, name: &str) -> Result<String, Error> {
        // Hash the collection name to create a unique filename.
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let filename = hasher.finish();

        let path = Path::new(&self.path)
            .join(COLLECTIONS_DIR)
            .join(filename.to_string())
            .to_str()
            .unwrap()
            .to_string();

        Ok(path)
    }

    /// Creates the collections directory on the path if it doesn't exist.
    fn setup_collections_dir(path: &str) -> Result<(), Error> {
        let collections_dir = Path::new(path).join(COLLECTIONS_DIR);
        if !collections_dir.exists() {
            create_dir_all(&collections_dir)?;
        }

        Ok(())
    }

    /// Returns the UNIX timestamp in milliseconds.
    fn get_timestamp(&self) -> usize {
        let now = SystemTime::now();
        // We can unwrap safely since UNIX_EPOCH is always valid.
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap();
        timestamp.as_millis() as usize
    }
}
