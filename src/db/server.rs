use instant_distance::HnswMap as HNSW;
use instant_distance::{Builder, Search};
use sled::Db as DB;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Data type for the key-value store value's metadata.
pub type Data = HashMap<String, String>;

// This is the data structure that will be stored in
// the key-value store as the value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub embedding: Vec<f32>,
    pub data: Data,
}

// Use Arc and Mutex to share the index across threads.
// Use Vector for the index to avoid mutating the index directly.
type Index = Arc<Mutex<Vec<HNSW<Value, String>>>>;

// Configuration for the database server.
pub struct Config {
    pub dimension: usize,
    pub token: String,
}

pub struct Server {
    pub config: Config,
    index: Index,
    db: DB, // Storage for key-value pairs.
}

impl Server {
    pub fn new(config: Config) -> Server {
        let index: Index = Arc::new(Mutex::new(Vec::with_capacity(1)));
        let db: DB = sled::open("data").unwrap();
        Server { index, config, db }
    }

    // Native functionality handler.
    // These are the functions that handle the native
    // functionality of the database.
    // Example: get, set, delete, etc.

    pub fn get(&self, key: String) -> Result<Value, &str> {
        // Check if the key exists.
        if !self.db.contains_key(key.clone()).unwrap() {
            return Err("The value is not found.");
        }

        let value = self.db.get(key).unwrap().unwrap();
        Ok(serde_json::from_slice(&value).unwrap())
    }

    pub fn set(&self, key: String, value: Value) -> Result<Value, &str> {
        // Validate the dimension of the value.
        if value.embedding.len() != self.config.dimension {
            return Err("The embedding dimension is invalid.");
        }

        let result = {
            let key = key.clone();
            let value = serde_json::to_vec(&value).unwrap();
            self.db.insert(key, value)
        };

        if result.is_err() {
            return Err("Error when setting the value.");
        }

        Ok(value)
    }

    pub fn delete(&self, key: String) -> Result<Value, &str> {
        // Check if the key exists.
        if !self.db.contains_key(key.clone()).unwrap() {
            return Err("The key does not exist.");
        }

        let result = {
            let value = self.db.remove(key.clone()).unwrap().unwrap();
            serde_json::from_slice(&value)
        };

        match result {
            Ok(value) => Ok(value),
            Err(_) => Err("Unable to remove the key."),
        }
    }

    // Index functionality handler.
    // Functions that handle the indexing of the database.

    pub fn build(
        &self,
        ef_search: usize,
        ef_construction: usize,
    ) -> Result<&str, &str> {
        // Clear the current index.
        // This makes sure that the index is built from scratch
        // and accomodate changes made to the key-value store.
        let mut index = self.index.lock().unwrap();
        index.clear();

        // Separate key-value to keys and values.
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for result in self.db.iter() {
            let (key, value) = result.unwrap();
            let key = String::from_utf8_lossy(&key).to_string();
            let value: Value = serde_json::from_slice(&value).unwrap();
            keys.push(key);
            values.push(value);
        }

        // Build the index.
        let _index = Builder::default()
            .ef_search(ef_search)
            .ef_construction(ef_construction)
            .build(values, keys);

        index.push(_index);
        Ok("The index is built successfully.")
    }

    pub fn search(
        &self,
        embedding: Vec<f32>,
        count: usize,
    ) -> Result<Vec<Data>, &str> {
        // Validate the dimension of the embedding.
        if embedding.len() != self.config.dimension {
            return Err("The embedding dimension is invalid.");
        }

        // Get the index or return error if it's not built.
        let _index = self.index.lock().unwrap();
        let index = match _index.first() {
            Some(index) => index,
            None => return Err("The index is not built yet."),
        };

        // Create a decoy value with the provided embedding.
        // Data is not needed for the search.
        let point = Value { embedding, data: HashMap::new() };

        // Search the index.
        let mut search = Search::default();
        let results = index.search(&point, &mut search);

        // Get the keys from the result.
        let mut data: Vec<Data> = Vec::new();
        for result in results {
            let value = result.point;
            data.push(value.data.clone());
        }

        // Truncate the result to count.
        data.truncate(count);

        Ok(data)
    }
}

// This is the implementation of the Point trait.
// This is needed by the library to calculate the distance
// between two vectors.
impl instant_distance::Point for Value {
    fn distance(&self, other: &Self) -> f32 {
        let mut sum = 0.0;

        // Implement Euclidean distance formula.
        // https://en.wikipedia.org/wiki/Euclidean_distance
        for i in 0..self.embedding.len().min(other.embedding.len()) {
            sum += (self.embedding[i] - other.embedding[i]).powi(2);
        }

        sum.sqrt()
    }
}
