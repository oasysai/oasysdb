use super::*;

/// The database storing vector collections.
pub struct Database {
    collections: Db,
    count: usize,
}

impl Database {
    /// Re-creates and opens the database at the given path.
    /// This method will delete the database if it exists.
    /// * `path` - Directory to store the database.
    pub fn new(path: &str) -> Self {
        // Remove the database dir if it exists.
        if Path::new(path).exists() {
            remove_dir_all(path).unwrap();
        }

        // Using sled::Config to prevent name collisions.
        let config = sled::Config::new().path(path);
        let collections = config.open().unwrap();
        Self { collections, count: 0 }
    }

    /// Opens existing or creates new database.
    /// If the database doesn't exist, it will be created.
    /// * `path` - Directory to store the database.
    pub fn open(path: &str) -> Self {
        let collections = sled::open(path).unwrap();
        let count = collections.len();
        Self { collections, count }
    }

    /// Creates a new collection in the database.
    /// * `name` - Name of the collection.
    /// * `config` - Collection configuration. Uses default if none.
    /// * `records` - Vector records to insert into the collection.
    pub fn create_collection<D, const N: usize, const M: usize>(
        &mut self,
        name: &str,
        config: Option<&Config>,
        records: Option<&[Record<D, N>]>,
    ) -> Collection<D, N, M>
    where
        D: Copy + Serialize,
    {
        // This prevents the variable from being dropped.
        let default_config = Config::default();

        let config = match config {
            Some(config) => config,
            None => &default_config,
        };

        // Create new or build a collection.
        let collection = match records {
            Some(records) => Collection::build(config, records),
            None => Collection::new(config),
        };

        self.save_collection(name, &collection);
        collection
    }

    /// Gets a collection from the database.
    /// * `name` - Name of the collection.
    pub fn get_collection<D, const N: usize, const M: usize>(
        &self,
        name: &str,
    ) -> Collection<D, N, M>
    where
        D: Copy + Serialize + DeserializeOwned,
    {
        let value = self.collections.get(name).unwrap().unwrap();
        bincode::deserialize(&value).unwrap()
    }

    /// Saves new or update existing collection to the database.
    /// * `name` - Name of the collection.
    /// * `collection` - Vector collection to save.
    pub fn save_collection<D, const N: usize, const M: usize>(
        &mut self,
        name: &str,
        collection: &Collection<D, N, M>,
    ) where
        D: Copy + Serialize,
    {
        let mut new = false;

        // Check if it's a new collection.
        if !self.collections.contains_key(name).unwrap() {
            new = true;
        }

        let value = bincode::serialize(collection).unwrap();
        self.collections.insert(name, value).unwrap();

        // If it's a new collection, update the count.
        if new {
            self.count += 1;
        }
    }

    /// Deletes a collection from the database.
    /// * `name` - Collection name to delete.
    pub fn delete_collection(&mut self, name: &str) {
        self.collections.remove(name).unwrap();
        self.count -= 1;
    }

    /// Returns the number of collections in the database.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the database is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
