use super::*;

/// The database storing vector collections.
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.database"))]
pub struct Database {
    collections: Db,
    count: usize,
}

#[cfg_attr(feature = "py", pymethods)]
impl Database {
    #[cfg(feature = "py")]
    #[staticmethod]
    #[pyo3(name = "new")]
    fn py_new(path: &str) -> PyResult<Self> {
        Self::new(path).map_err(|e| e.into())
    }

    #[cfg(feature = "py")]
    #[new]
    fn py_open(path: &str) -> PyResult<Self> {
        Self::open(path).map_err(|e| e.into())
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

    #[cfg(feature = "py")]
    fn __len__(&self) -> usize {
        self.len()
    }
}

impl Database {
    /// Re-creates and opens the database at the given path.
    /// This method will delete the database if it exists.
    /// * `path` - Directory to store the database.
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
    pub fn open(path: &str) -> Result<Self, Error> {
        let collections = sled::open(path)?;
        let count = collections.len();
        Ok(Self { collections, count })
    }
}
