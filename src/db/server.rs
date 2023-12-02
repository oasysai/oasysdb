use hora::core::ann_index::ANNIndex;
use hora::index::hnsw_idx::HNSWIndex;
use hora::index::hnsw_params::HNSWParams;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

// Import route handlers.
use super::routes::kvs;
use super::routes::root;
use super::routes::version;

// Import utils.
use super::utils::response as res;
use super::utils::stream;

// This is the data structure that will be stored in
// the key-value store as the value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub embedding: Vec<f32>,
    pub data: HashMap<String, String>,
}

// Use Arc and Mutex to share the key-value store
// across threads while ensuring exclusive access.
type KeyValue = Arc<Mutex<HashMap<String, Value>>>;

// HNSW index used for NN search.
// Using f32 which refers to the inner type of the embedding.
// Using String which refers to the type of the key.
type Index = HNSWIndex<f32, String>;

pub struct Server {
    addr: SocketAddr,
    kvs: KeyValue,
    index: Index,
}

impl Server {
    pub async fn new(host: &str, port: &str, dimension: usize) -> Server {
        let addr = format!("{}:{}", host, port).parse().unwrap();

        // Initialize a new key-value store.
        let kvs = Arc::new(Mutex::new(HashMap::new()));

        // Create a new HNSW index.
        let index_params = HNSWParams::<f32>::default();
        let index: Index = HNSWIndex::new(dimension, &index_params);

        Server { addr, kvs, index }
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
                let mut _res_body = HashMap::new();
                _res_body.insert("error", "Invalid request.");
                let res = res::create_response(400, Some(_res_body));
                stream::write(&mut stream, res).await;
                break;
            }

            // Unwrap the data.
            let req = _req.as_ref().unwrap();
            let route = req.route.clone();

            // Get response based on different routes and methods.
            let response = match route.as_str() {
                "/" => root::handler(req),
                "/version" => version::handler(req),
                _ if route.starts_with("/kvs") => kvs::handler(self, req),
                _ => res::get_not_found_response(None),
            };

            // Write the data back to the client.
            stream::write(&mut stream, response).await;
        }
    }

    // Native functionality handler.
    // These are the functions that handle the native
    // functionality of the database.
    // Example: get, set, delete, etc.

    pub fn get(&self, key: String) -> Option<Value> {
        let kvs = self.kvs.lock().unwrap();
        kvs.get(&key).cloned()
    }

    pub fn set(&mut self, key: String, value: Value) -> Result<Value, &str> {
        // Add the key-value to the index.
        let embedding = value.embedding.clone();
        let res = self.index.add(&embedding, key.clone());

        // Handle error when adding embedding to index.
        if res.is_err() {
            let _err = res.err().unwrap();

            // Handle hora default error message.
            let message = match _err {
                _ if _err.contains("dimension") => "The embedding dimension is invalid.",
                _ => "Unable to add the value embedding.",
            };

            return Err(message);
        }

        // Add the key-value to the store.
        let mut kvs = self.kvs.lock().unwrap();
        kvs.insert(key, value.clone());

        Ok(value)
    }

    pub fn delete(&self, key: String) {
        let mut kvs = self.kvs.lock().unwrap();
        kvs.remove(&key);
    }
}
