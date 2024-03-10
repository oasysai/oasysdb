use super::*;

/// The database storing vector collections.
#[pyclass(module = "oasysdb.database")]
pub struct Database {
    collections: Db,
    count: usize,
}

#[pymethods]
impl Database {
    /// Re-creates and opens the database at the given path.
    /// This method will delete the database if it exists.
    /// * `path` - Directory to store the database.
    #[staticmethod]
    pub fn new(path: &str) -> Result<Self, Error> {
        // Remove the database dir if it exists.
        if Path::new(path).exists() {
            remove_dir_all(path)?;
        }

        // Using sled::Config to prevent name collisions
        // with collection's Config.
        let config = sled::Config::new().path(path);
        let collections = config.open()?;
        Ok(Self { collections, count: 0 })
    }

    /// Opens existing or creates new database.
    /// If the database doesn't exist, it will be created.
    /// * `path` - Directory to store the database.
    #[new]
    pub fn open(path: &str) -> Result<Self, Error> {
        let collections = sled::open(path)?;
        let count = collections.len();
        Ok(Self { collections, count })
    }

    /// Creates a new collection in the database.
    /// * `name` - Name of the collection.
    /// * `config` - Collection configuration. Uses default if none.
    /// * `records` - Vector records to insert into the collection.
    pub fn create_collection(
        &mut self,
        name: &str,
        config: Option<&Config>,
        records: Option<Vec<Record>>,
    ) -> Result<Collection, Error> {
        // This prevents the variable from being dropped.
        let default_config = Config::default();

        let config = match config {
            Some(config) => config,
            None => &default_config,
        };

        // Create new or build a collection.
        let collection = match records {
            Some(records) => Collection::build(config, records)?,
            None => Collection::new(config),
        };

        self.save_collection(name, &collection)?;
        Ok(collection)
    }

    /// Gets a collection from the database.
    /// * `name` - Name of the collection.
    pub fn get_collection(&self, name: &str) -> Result<Collection, Error> {
        let value = self.collections.get(name)?;
        match value {
            Some(value) => Ok(bincode::deserialize(&value)?),
            None => Err(Error::collection_not_found()),
        }
    }

    /// Saves new or update existing collection to the database.
    /// * `name` - Name of the collection.
    /// * `collection` - Vector collection to save.
    pub fn save_collection(
        &mut self,
        name: &str,
        collection: &Collection,
    ) -> Result<(), Error> {
        let mut new = false;

        // Check if it's a new collection.
        if !self.collections.contains_key(name)? {
            new = true;
        }

        let value = bincode::serialize(collection)?;
        self.collections.insert(name, value)?;

        // If it's a new collection, update the count.
        if new {
            self.count += 1;
        }

        Ok(())
    }

    /// Deletes a collection from the database.
    /// * `name` - Collection name to delete.
    pub fn delete_collection(&mut self, name: &str) -> Result<(), Error> {
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

    fn __len__(&self) -> usize {
        self.len()
    }
}
