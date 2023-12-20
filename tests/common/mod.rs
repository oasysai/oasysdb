use oasysdb::db::routes::handle_request;
use oasysdb::db::server::{Config, Server, Value};
use rand::random;
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use std::fs::remove_dir_all;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

// Directory used to persist data for testing.
// This directory will be removed after the tests.
const DATA_DIR: &str = "tests/data";

pub async fn run_server(port: String) -> Runtime {
    // Create a new Tokio runtime.
    // This runtime will be used by the server.
    let runtime = Runtime::new().unwrap();

    // Start the server in the runtime.
    runtime.spawn(async move {
        // Create a TCP listener to accept connections.
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let (mut stream, _) = listener.accept().await.unwrap();

        // Server configuration for testing only.
        let config = {
            let dimension = 2;
            let token = "token".to_string();
            let path = format!("{}/{}", DATA_DIR, port);
            Config { dimension, token, path }
        };

        // Create a new server.
        let server = Server::new(config);

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

        // Build the graph.
        let ef = 10; // Use small EF for testing only.
        server.build("default".into(), ef, ef).unwrap();

        // Start the server.
        handle_request(&server, &mut stream).await;
    });

    // Return runtime as a handle to stop the server.
    runtime
}

pub async fn stop_server(runtime: Runtime, port: String) {
    // Shutdown the runtime.
    runtime.shutdown_background();

    // Remove the test data directory.
    remove_dir_all(format!("{}/{}", DATA_DIR, port)).unwrap();
}

pub fn get_headers() -> HeaderMap {
    // Generate headers for the test requests.
    let mut headers = HeaderMap::new();
    headers.insert("x-oasysdb-token", "token".parse().unwrap());
    headers
}
