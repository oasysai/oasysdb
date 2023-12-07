use instant_distance::HnswMap as HNSW;
use instant_distance::{Builder, Search};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

// Import route handlers.
use super::routes::build;
use super::routes::kvs;
use super::routes::root;
use super::routes::search;
use super::routes::version;

// Import utils.
use super::utils::response as res;
use super::utils::stream;

// Data type for the key-value store value's metadata.
pub type Data = HashMap<String, String>;

// This is the data structure that will be stored in
// the key-value store as the value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub embedding: Vec<f32>,
    pub data: Data,
}

// Use Arc and Mutex to share the key-value store
// across threads while ensuring exclusive access.
type KeyValue = Arc<Mutex<HashMap<String, Value>>>;

// Use Option to allow the index to be None.
// None means that the index is not built yet.
type Index = Option<HNSW<Value, String>>;

// Configuration for the database server.
pub struct Config {
    pub dimension: usize,
}

pub struct Server {
    addr: SocketAddr,
    kvs: KeyValue,
    index: Index,
    config: Config,
}

impl Server {
    pub fn new(host: &str, port: &str, config: Config) -> Server {
        // Parse the host and port into a SocketAddr.
        let addr = format!("{}:{}", host, port).parse().unwrap();

        // Initialize a new key-value store.
        let kvs = Arc::new(Mutex::new(HashMap::new()));

        // Initialize index as None since it's not built yet.
        let index: Index = None;

        Server { addr, kvs, index, config }
    }

    pub async fn serve(&mut self) {
        // Bind a listener to the socket address.
        let listener = TcpListener::bind(self.addr).await.unwrap();

        // Accept and handle connections from clients.
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let handler = self._handle_connection(stream).await;
            tokio::spawn(async move { handler });
        }
    }

    async fn _handle_connection(&mut self, mut stream: TcpStream) {
        loop {
            // Read request from the client.
            let _req = stream::read(&mut stream).await;

            // Handle disconnection or invalid request.
            // Return invalid request response.
            if _req.is_none() {
                let res = res::get_error_response(400, "Invalid request.");
                stream::write(&mut stream, res).await;
                break;
            }

            // Unwrap the data.
            let req = _req.as_ref().unwrap();
            let route = req.route.clone();

            // Get response based on different routes and methods.
            let response = match route.as_str() {
                // Utils.
                "/" => root::handler(req),
                "/version" => version::handler(req),
                // Indexing.
                "/build" => build::handler(self, req),
                "/search" => search::handler(self, req),
                // Key-value store.
                _ if route.starts_with("/kvs") => kvs::handler(self, req),
                _ => res::get_404_response(),
            };

            // Write the data back to the client.
            stream::write(&mut stream, response).await;
        }
    }

    // Native functionality handler.
    // These are the functions that handle the native
    // functionality of the database.
    // Example: get, set, delete, etc.

    pub fn get(&self, key: String) -> Result<Value, &str> {
        let kvs = self.kvs.lock().unwrap();
        kvs.get(&key).cloned().ok_or("The value is not found.")
    }

    pub fn set(&mut self, key: String, value: Value) -> Result<Value, &str> {
        // Validate the dimension of the value.
        if value.embedding.len() != self.config.dimension {
            return Err("The embedding dimension is invalid.");
        }

        // Insert the key-value pair into the key-value store.
        let mut kvs = self.kvs.lock().unwrap();
        kvs.insert(key, value.clone());
        Ok(value)
    }

    pub fn delete(&self, key: String) -> Result<Value, &str> {
        let mut kvs = self.kvs.lock().unwrap();
        kvs.remove(&key).ok_or("The key doesn't exist.")
    }

    // Index functionality handler.
    // Functions that handle the indexing of the database.

    pub fn build(
        &mut self,
        ef_search: usize,
        ef_construction: usize,
    ) -> Result<&str, &str> {
        // Clear the current index.
        // This makes sure that the index is built from scratch
        // and accomodate changes made to the key-value store.
        self.index = None;

        // Get the key-value store.
        let kvs = self.kvs.lock().unwrap();

        // Separate key-value to keys and values.
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in kvs.iter() {
            keys.push(key.clone());
            values.push(value.clone());
        }

        // Build the index.
        let index = Builder::default()
            .ef_search(ef_search)
            .ef_construction(ef_construction)
            .build(values, keys);

        self.index = Some(index);
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
        let index = self.index.as_ref().ok_or("The index is not built.")?;

        // Create a decoy value with the provided embedding.
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
        for i in 0..self.embedding.len() {
            sum += (self.embedding[i] - other.embedding[i]).powi(2);
        }

        sum.sqrt()
    }
}
