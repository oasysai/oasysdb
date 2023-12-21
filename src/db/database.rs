use instant_distance::HnswMap as HNSW;
use instant_distance::*;
use serde::*;
use sled::Db as DB;
use std::collections::HashMap;

type Error = &'static str;

pub type Data = HashMap<String, String>;
pub type Embedding = Vec<f32>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Value {
    pub embedding: Embedding,
    pub data: Data,
}

pub type Graph = HNSW<Value, String>;

#[derive(Serialize, Deserialize)]
pub struct GraphConfig {
    pub name: String,
    pub ef_construction: usize,
    pub ef_search: usize,
}

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

    pub fn get_value(&self, key: &str) -> Result<Value, Error> {
        let result = self.value_db.get(key);

        // Making sure we don't panic if error when retrieving value.
        if result.is_err() {
            return Err("Failed to get value.");
        }

        match result.unwrap() {
            Some(value) => Ok(serde_json::from_slice(&value).unwrap()),
            None => Err("Value not found."),
        }
    }

    pub fn set_value(&self, key: &str, value: Value) -> Result<(), Error> {
        // Validate that the value has the correct dimension.
        if value.embedding.len() != self.config.dimension {
            return Err("Invalid embedding dimension.");
        }

        // Serialize the value to bytes.
        let value = serde_json::to_vec(&value).unwrap();

        match self.value_db.insert(key, value) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to set value."),
        }
    }

    pub fn delete_value(&self, key: &str) -> Result<(), Error> {
        match self.value_db.remove(key).unwrap() {
            Some(_) => Ok(()),
            None => Err("Value not found."),
        }
    }

    // Graph methods.

    pub fn create_graph(&self, config: GraphConfig) -> Result<(), Error> {
        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        // Iterate over all values in the database and separate
        // them into keys and values.
        for result in self.value_db.iter() {
            let (key, value) = result.unwrap();
            keys.push(String::from_utf8_lossy(&key).to_string());
            values.push(serde_json::from_slice(&value).unwrap());
        }

        // Build the HNSW graph with the given config.
        let graph = Builder::default()
            .ef_construction(config.ef_construction)
            .ef_search(config.ef_search)
            .build(values, keys);

        // Serialize the graph to bytes to store in the database.
        let graph = serde_json::to_vec(&graph).unwrap();

        match self.graph_db.insert(config.name, graph) {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to create graph."),
        }
    }

    pub fn delete_graph(&self, name: &str) -> Result<(), Error> {
        match self.graph_db.remove(name).unwrap() {
            Some(_) => Ok(()),
            None => Err("Graph not found."),
        }
    }

    pub fn query_graph(
        &self,
        name: &str,
        embedding: Embedding,
        k: usize,
    ) -> Result<Vec<Data>, Error> {
        // Validate embedding dimension.
        if embedding.len() != self.config.dimension {
            return Err("Invalid embedding dimension.");
        }

        let get_graph = self.graph_db.get(name);

        // Make sure we don't panic if error when retrieving graph.
        if get_graph.is_err() {
            return Err("Failed to get graph.");
        }

        let graph = match get_graph.ok().unwrap() {
            Some(graph) => graph,
            None => return Err("Graph not found."),
        };

        // Deserialize the graph.
        let graph: Graph = serde_json::from_slice(&graph).unwrap();

        // Decoy value with the provided embedding.
        // Data is not needed for the query process.
        let point = Value { embedding, data: HashMap::new() };

        // Query the graph.
        let mut query = Search::default();
        let results = graph.search(&point, &mut query);

        let mut data: Vec<Data> = Vec::new();
        for result in results {
            let value = result.point;
            data.push(value.data.clone());
        }

        data.truncate(k);
        Ok(data)
    }
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
