use oasysdb::db::server::Server;
use tokio::runtime::Runtime;

// The host and port to bind to for testing.
pub const HOST: &str = "127.0.0.1";
pub const PORT: &str = "31415";

pub async fn run_server() -> Runtime {
    // Create a new Tokio runtime.
    // This runtime will be used by the server.
    let runtime = Runtime::new().unwrap();

    // Start the server in the runtime.
    runtime.spawn(async move {
        let server = Server::new(HOST, PORT).await;
        server.serve().await;
    });

    runtime
}

pub async fn stop_server(runtime: Runtime) {
    // Shutdown the runtime.
    runtime.shutdown_background();
}
