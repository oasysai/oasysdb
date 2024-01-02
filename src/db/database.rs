use instant_distance::HnswMap as HNSW;
use instant_distance::*;
use serde::*;
use sled::Db as DB;
use std::collections::HashMap;

/// The data structure of the value that will be
/// stored in the value database.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Value {
    /// The vector embedding of the data. The dimension of the embedding
    /// must match the dimension configured when initializing the database.
    pub embedding: Embedding,
    /// The data associated with the embedding. This can be used to store
    /// any data that is associated with the embedding. For example, if
    /// the embedding is a vector of a person's face, the data can be
    /// the person's name.
    pub data: Data,
}

/// Index graph configuration.
/// Note that for EF parameters, the higher the number of this parameters,
/// the more accurate the graph will be but the slower it will be to build.
#[derive(Serialize, Deserialize, Clone)]
pub struct GraphConfig {
    /// The name of the graph. Also used to identify the
    /// graph in the graph database.
    pub name: String,
    /// The number of neighbors that will be calculated
    /// during the construction of the graph.
    pub ef_construction: usize,
    /// The number of neighbors that will be considered
    /// when searching the nearest neighbors in the graph.
    pub ef_search: usize,
    /// Optional hashmap of data that will be used to filter
    /// the values used to build the graph. Works like a WHERE
    /// clause in SQL with an AND operator. Only values that
    /// match all of the filter will be used to build the graph.
    pub filter: Option<Data>,
}

/// The value that will be stored in the graph database.
#[derive(Serialize, Deserialize)]
struct GraphStore {
    graph: Graph,
    config: GraphConfig,
}

/// The configuration of the database.
pub struct Config {
    /// The path where the database will be persisted.
    pub path: String,
    /// The dimension of the embeddings that will be stored.
    /// This is used to validate that the embeddings supplied
    /// have the correct dimension.
    pub dimension: usize,
}

/// The vector database. It contains the configuration as well as the value
/// database and the graph database. The value database is used to store the
/// `Value` and the graph database is used to store the `GraphStore`.
pub struct Database {
    pub config: Config,
    value_db: DB,
    graph_db: DB,
}

impl Database {
    /// Creates a new database with the given configuration. The value
    /// database will be persisted in `/<path>/values` and the graph
    /// database will be persisted in `/<path>/graphs`.
    pub fn new(config: Config) -> Database {
        let value_db = sled::open(format!("{}/values", config.path)).unwrap();
        let graph_db = sled::open(format!("{}/graphs", config.path)).unwrap();
        Database { config, value_db, graph_db }
    }

    // Value database methods.

    /// Gets the value with the given key from the value database. If the
    /// value doesn't exist, it will return an error.
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

    /// Sets the value with the given key in the value database.
    /// If the value already exists, it will be overwritten with
    /// the new value.
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

    /// Deletes the value from the value database.
    pub fn delete_value(&self, key: &str) -> Result<(), Error> {
        match self.value_db.remove(key).unwrap() {
            Some(_) => Ok(()),
            None => Err("Value not found."),
        }
    }

    /// Resets the value database. This method will delete all values
    /// in the value database.
    pub fn reset_values(&self) -> Result<(), Error> {
        match self.value_db.clear() {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to reset values."),
        }
    }

    // Graph methods.

    /// Creates a graph with the given configuration. This will create a
    /// graph from the key-values. The graph will be serialized and stored
    /// in the graph database.
    ///
    /// Once built, the values used to build the graph is persisted inside
    /// of the graph like a snapshot. This means that when a value is added
    /// or deleted, the graph needs to be recreated to reflect the changes.
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

    /// Deletes the graph with the given name from the graph database.
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

    /// Resets the graph database. This method will delete all graphs
    /// in the graph database.
    pub fn reset_graphs(&self) -> Result<(), Error> {
        match self.graph_db.clear() {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to reset graphs."),
        }
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

// Type aliases for readability.
type Error = &'static str;
type Data = HashMap<String, String>;
type Embedding = Vec<f32>;

/// A type alias for the HNSW (Hierarchical Navigable Small World)
/// graph. This is the graph that will be used to query the embedding.
/// Check the documentation of `instant_distance` for more info:
/// https://github.com/instant-labs/instant-distance
type Graph = HNSW<Value, String>;
