use oasysdb::db::server::{Config, Server, Value};
use rand::random;
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use tokio::runtime::Runtime;

pub async fn run_server(port: String) -> Runtime {
    // Create a new Tokio runtime.
    // This runtime will be used by the server.
    let runtime = Runtime::new().unwrap();

    // Start the server in the runtime.
    runtime.spawn(async move {
        // Server parameters.
        let host = "127.0.0.1";
        let port = port.as_str();

        // Server configuration.
        let config = Config { dimension: 2, token: "token".to_string() };

        // Create a new server.
        let mut server = Server::new(host, port, config);

        // Pre-populate the key-value store.
        for i in 0..9 {
            // Generate value with random embeddings.
            let value = Value {
                embedding: vec![random::<f32>(); 2],
                data: HashMap::new(),
            };

            // Set the key-value pair.
            let key = format!("key-{}", i);
            server.set(key, value).unwrap();
        }

        // Build the index.
        let ef = 10; // Use small EF for testing only.
        server.build(ef, ef).unwrap();

        // Start the server.
        server.serve().await;
    });

    // Return runtime as a handle to stop the server.
    runtime
}

pub async fn stop_server(runtime: Runtime) {
    // Shutdown the runtime.
    runtime.shutdown_background();
}

pub fn get_headers() -> HeaderMap {
    // Generate headers for the test requests.
    let mut headers = HeaderMap::new();
    headers.insert("x-oasysdb-token", "token".parse().unwrap());
    headers
}
