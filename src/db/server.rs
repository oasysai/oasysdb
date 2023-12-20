use instant_distance::HnswMap as HNSW;
use instant_distance::{Builder, Search};
use sled::Db as Database;
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

// Use Arc and Mutex to share the graphs across threads.
// Store the graphs in a HashMap to allow multiple graphs.
type Graph = HNSW<Value, String>;
type Graphs = Arc<Mutex<HashMap<String, Graph>>>;

// Configuration for the database server.
pub struct Config {
    pub dimension: usize,
    pub token: String,
    pub path: String,
}

pub struct Server {
    pub config: Config,
    graphs: Graphs,
    graph_db: Database,
    value_db: Database,
}

impl Server {
    pub fn new(config: Config) -> Server {
        // Load the key-value store.
        let value_db = {
            let path = format!("{}/values", config.path.clone());
            sled::open(path).unwrap()
        };

        // Load the graph data.
        // This is used to restore the graphs to memory
        // when the server is restarted.
        let graph_db = {
            let path = format!("{}/graphs", config.path.clone());
            sled::open(path).unwrap()
        };

        // Create a new graphs HashMap.
        let graphs: Graphs = Arc::new(Mutex::new(HashMap::new()));

        // Restore the graphs from the database.
        for item in graph_db.iter() {
            let (name, graph) = item.unwrap();
            let name = String::from_utf8_lossy(&name).to_string();
            let graph: Graph = serde_json::from_slice(&graph).unwrap();
            graphs.lock().unwrap().insert(name, graph);
        }

        Server { config, graphs, graph_db, value_db }
    }

    // Native functionality handler.
    // These are the functions that handle the native
    // functionality of the database.
    // Example: get, set, delete, etc.

    pub fn get(&self, key: String) -> Result<Value, &str> {
        // Check if the key exists.
        if !self.value_db.contains_key(key.clone()).unwrap() {
            return Err("The value is not found.");
        }

        let value = self.value_db.get(key).unwrap().unwrap();
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
            self.value_db.insert(key, value)
        };

        if result.is_err() {
            return Err("Error when setting the value.");
        }

        Ok(value)
    }

    pub fn delete(&self, key: String) -> Result<Value, &str> {
        // Check if the key exists.
        if !self.value_db.contains_key(key.clone()).unwrap() {
            return Err("The key does not exist.");
        }

        let result = {
            let value = self.value_db.remove(key.clone()).unwrap().unwrap();
            serde_json::from_slice(&value)
        };

        match result {
            Ok(value) => Ok(value),
            Err(_) => Err("Unable to remove the key."),
        }
    }

    // Graphs functionality handlers.
    // This handles building and querying the graphs.

    pub fn build(
        &self,
        name: String,
        ef_search: usize,
        ef_construction: usize,
    ) -> Result<&str, &str> {
        // Separate key-value to keys and values.
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for result in self.value_db.iter() {
            let (key, value) = result.unwrap();
            let key = String::from_utf8_lossy(&key).to_string();
            let value: Value = serde_json::from_slice(&value).unwrap();
            keys.push(key);
            values.push(value);
        }

        // Build the graph.
        let new_graph: Graph = Builder::default()
            .ef_search(ef_search)
            .ef_construction(ef_construction)
            .build(values, keys);

        // Store the graph config to the database.
        // This allows the graph to be rebuilt when the server restarts.
        let graph_config = serde_json::to_vec(&new_graph).unwrap();
        self.graph_db.insert(name.clone(), graph_config).unwrap();

        // Store the graph to Server.graphs which exists in memory.
        let mut graphs = self.graphs.lock().unwrap();
        graphs.insert(name, new_graph);

        Ok("The graph is built successfully.")
    }

    pub fn query(
        &self,
        name: String, // Graph name.
        embedding: Vec<f32>,
        count: usize,
    ) -> Result<Vec<Data>, &str> {
        // Validate the dimension of the embedding.
        if embedding.len() != self.config.dimension {
            return Err("The embedding dimension is invalid.");
        }

        // Get the graph from the HashMap with the provided name.
        // Graph name = HashMap key.
        let graphs = self.graphs.lock().unwrap();
        let graph: &Graph = match graphs.get(&name) {
            Some(graph) => graph,
            None => return Err("The graph is not found."),
        };

        // Create a decoy value with the provided embedding.
        // Data is not needed for the query process.
        let point = Value { embedding, data: HashMap::new() };

        // Query the graph.
        let mut query = Search::default();
        let results = graph.search(&point, &mut query);

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
