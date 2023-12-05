use oasysdb::db::server::{Server, Value};
use rand::random;
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
    let _port = random_number.to_string();
    let port = _port.clone();

    // Start the server in the runtime.
    runtime.spawn(async move {
        // Server parameters.
        let host = "127.0.0.1";
        let port = port.as_str();

        // Create a new server.
        let mut server = Server::new(host, port).await;

        // Pre-populate the key-value store.
        for i in 0..9 {
            // Generate value with random embeddings.
            let value = Value {
                embedding: vec![random::<f32>(), random::<f32>()],
                data: HashMap::new(),
            };

            // Set the key-value pair.
            let key = format!("key-{}", i);
            server.set(key, value).unwrap();
        }

        // Start the server.
        server.serve().await;
    });

    // Use runtime as a handle to stop the server.
    // Use port to make requests to the server.
    (runtime, _port)
}

pub async fn stop_server(runtime: Runtime) {
    // Shutdown the runtime.
    runtime.shutdown_background();
}
