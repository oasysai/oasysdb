use instant_distance::HnswMap as HNSW;
use serde::{Deserialize, Serialize};
use sled::Db as DB;
use std::collections::HashMap;

pub struct Message(pub &'static str);

pub type Data = HashMap<String, String>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Value {
    pub embedding: Vec<f32>,
    pub data: Data,
}

type Graph = HNSW<Value, String>;

pub struct Config {
    pub path: String,
    pub dimension: usize,
}

pub struct Database {
    pub config: Config,
    value_db: DB,
    graph_db: DB,
}

impl Database {
    pub fn new(config: Config) -> Database {
        let value_db = sled::open(format!("{}/values", config.path)).unwrap();
        let graph_db = sled::open(format!("{}/graphs", config.path)).unwrap();
        Database { config, value_db, graph_db }
    }

    // Key-value store methods.

    pub fn get_value(&self, key: &str) -> Result<Value, Message> {
        let result = self.value_db.get(key);

        // Making sure we don't panic if error when retrieving value.
        if result.is_err() {
            return Err(Message("Failed to get value."));
        }

        match result.unwrap() {
            Some(value) => Ok(serde_json::from_slice(&value).unwrap()),
            None => Err(Message("Value not found.")),
        }
    }

    pub fn set_value(&self, key: &str, value: Value) -> Result<(), Message> {
        // Validate that the value has the correct dimension.
        if value.embedding.len() != self.config.dimension {
            return Err(Message("Invalid embedding dimension."));
        }

        let result = {
            let value = serde_json::to_vec(&value).unwrap();
            self.value_db.insert(key, value)
        };

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Message("Failed to set value.")),
        }
    }

    pub fn delete_value() {}

    // Graph methods.

    pub fn create_graph() {}

    pub fn delete_graph() {}

    pub fn query_graph() {}
}

// Implementation of the Point trait needed by the instant_distance
// crate to calculate the distance between two vectors.
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
