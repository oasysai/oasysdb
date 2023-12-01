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

pub struct Server {
    addr: SocketAddr,
    kvs: KeyValue,
}

impl Server {
    pub async fn new(host: &str, port: &str) -> Server {
        let addr = format!("{}:{}", host, port).parse().unwrap();
        let kvs = Arc::new(Mutex::new(HashMap::new()));
        Server { addr, kvs }
    }

    pub async fn serve(&self) {
        // Bind a listener to the socket address.
        let listener = TcpListener::bind(self.addr).await.unwrap();

        // Accept and handle connections from clients.
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let handler = self._handle_connection(stream).await;
            tokio::spawn(async move { handler });
        }
    }

    async fn _handle_connection(&self, mut stream: TcpStream) {
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

    pub fn set(&self, key: String, value: Value) {
        let mut kvs = self.kvs.lock().unwrap();
        kvs.insert(key, value);
    }

    pub fn delete(&self, key: String) {
        let mut kvs = self.kvs.lock().unwrap();
        kvs.remove(&key);
    }
}
