use oasysdb::db::server::{Server, Value};
use std::collections::HashMap;
use tokio::runtime::Runtime;

pub async fn run_server() -> (Runtime, String) {
    // Create a new Tokio runtime.
    // This runtime will be used by the server.
    let runtime = Runtime::new().unwrap();

    // Generate a random port: 314xx.
    // This is needed to run multiple tests in parallel and
    // prevent connection reset error when testing.
    let random_number = rand::random::<u16>() % 100 + 31400;
    let port = random_number.to_string();
    let _port = port.clone();

    // Start the server in the runtime.
    runtime.spawn(async move {
        let server = Server::new("127.0.0.1", _port.as_str()).await;

        // Define the initial key value.
        let value = Value {
            embedding: vec![0.0, 0.0, 0.0],
            data: HashMap::new(),
        };

        // Add initial key-value stores.
        server.set("initial_key".to_string(), value);

        // Start the server.
        server.serve().await;
    });

    // Use runtime as a handle to stop the server.
    // Use port to make requests to the server.
    (runtime, port)
}

pub async fn stop_server(runtime: Runtime) {
    // Shutdown the runtime.
    runtime.shutdown_background();
}
