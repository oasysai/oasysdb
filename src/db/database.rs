use instant_distance::HnswMap as HNSW;
use instant_distance::*;
use serde::*;
use sled::Db as DB;
use std::collections::HashMap;

type Error = &'static str;

pub type Data = HashMap<String, String>;
pub type Embedding = Vec<f32>;

/// A struct that represents a value that will be stored
/// in the key-value store of the database. The embedding
/// dimension must match the dimension set by the
/// `OASYSDB_DIMENSION` environment variable.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Value {
    pub embedding: Embedding,
    pub data: Data,
}

/// A type alias for the HNSW (Hierarchical Navigable Small World)
/// graph. This is the graph that will be used to query the embedding.
/// Check the documentation of `instant_distance` for more info:
/// https://github.com/instant-labs/instant-distance
pub type Graph = HNSW<Value, String>;

/// A struct that represents the configuration of a graph. This is
/// how the graph will be built and stored in the graph database.
///
/// `ef_construction` is the number of neighbors that will be used to
/// build the graph. `ef_search` is the number of neighbors that will
/// be used to search the graph. The higher the number of this parameters,
/// the more accurate the graph will be but the slower it will be.
#[derive(Serialize, Deserialize, Clone)]
pub struct GraphConfig {
    pub name: String,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub filter: Option<Data>,
}

/// A struct that represents the data that will be stored in the graph
/// database as a value. It contains the graph itself and the
/// configuration of the graph.
#[derive(Serialize, Deserialize)]
pub struct GraphStore {
    pub graph: Graph,
    pub config: GraphConfig,
}

/// A struct that represents the configuration of the database.
/// - `path`: The path where the database will be persisted.
/// - `dimension`: The dimension of the embeddings that will be stored.
///     This needs to be set by the `OASYSDB_DIMENSION` environment
///     variable and it is used to validate that the embeddings have the
///     correct dimension.
pub struct Config {
    pub path: String,
    pub dimension: usize,
}

/// A struct that represents the database. It contains the configuration
/// as well as the key-value store and the graph database. The key-value
/// store is used to store the `Value` and the graph database is used to
/// store the serialized graphs. This graph then can be deserialized and
/// queried to get the nearest neighbors of a given embedding.
pub struct Database {
    pub config: Config,
    value_db: DB,
    graph_db: DB,
}

impl Database {
    /// Creates a new database with the given configuration. The value
    /// database will be stored in `/<path>/values` and the graph database
    /// will be stored in `/<path>/graphs`.
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

    /// Creates a graph with the given configuration. This will create a
    /// graph from the key-values. The graph will be serialized and stored
    /// in the graph database.
    ///
    /// Unfortunatelly, the graph doesn't automatically update when a value
    /// is added or deleted. This means that a value is added or deleted,
    /// the graph needs to be recreated.
    pub fn create_graph(&self, config: GraphConfig) -> Result<(), Error> {
        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        // Check if the graph need a filter.
        let has_filter = config.filter.is_some();

        // Iterate over all values in the database and separate
        // them into keys and values.
        for result in self.value_db.iter() {
            let (key, value) = result.unwrap();
            let value: Value = serde_json::from_slice(&value).unwrap();

            // Filter the values as provided.
            // If the value doesn't match the filter, skip it.
            if has_filter {
                let filter: &Data = &config.filter.clone().unwrap();
                if !filter_data_match(&value.data, filter) {
                    continue;
                }
            }

            keys.push(String::from_utf8_lossy(&key).to_string());
            values.push(value);
        }

        // Build the HNSW graph with the given config.
        let graph = Builder::default()
            .ef_construction(config.ef_construction)
            .ef_search(config.ef_search)
            .build(values, keys);

        // Serialize data of the graph and config to
        // store in the database.
        let data = {
            let config = config.clone();
            let _data = GraphStore { graph, config };
            serde_json::to_vec(&_data).unwrap()
        };

        match self.graph_db.insert(config.name, data) {
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

    /// Queries the graph with the given name and returns the nearest
    /// neighbors of the given embedding. This doesn't return the
    /// `Value.embedding` but only the associated `Value.data`.
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

        let get_store = self.graph_db.get(name);

        // Make sure we don't panic if error when retrieving graph.
        if get_store.is_err() {
            return Err("Failed to get graph.");
        }

        let graph_store = match get_store.ok().unwrap() {
            Some(store) => store,
            None => return Err("Graph not found."),
        };

        // Deserialize the graph store.
        let graph_store: GraphStore =
            serde_json::from_slice(&graph_store).unwrap();

        // Decoy value with the provided embedding.
        // Data is not needed for the query process.
        let point = Value { embedding, data: HashMap::new() };

        // Query the graph.
        let graph = graph_store.graph;
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

/// Checks if the given data matches the given filter. Iterates over the
/// filter to check if any of the data matches. Only if the value matches
/// over all of the filter, it will return true.
fn filter_data_match(data: &Data, filter: &Data) -> bool {
    for (key, value) in filter {
        if data.get(key).unwrap() != value {
            return false;
        }
    }

    true
}
