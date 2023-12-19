use dotenv::dotenv;
use oasysdb::db::server::Config;
use oasysdb::serve;
use std::env;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file.
    // This is only needed for development.
    dotenv().ok();

    // Port where OasysDB will listen for connections.
    let port = env::var("OASYSDB_PORT").unwrap_or(String::from("3141"));

    // The embedding dimension of the database. Required.
    let dimension = env_get_dimension();

    // Create the server configuration.
    let config = {
        // The token used to authenticate requests to the server.
        let token = get_env("OASYSDB_TOKEN");
        let path = "data".to_string();
        Config { dimension, token, path }
    };

    // Display the server configuration.
    println!("OasysDB is running on port {}.", port);
    println!("OasysDB accepts embeddings of {} dimension.", dimension);

    // Create and start the server.
    let host = "0.0.0.0";
    serve(host, &port, config).await;
}

// Utility functions.
// Helper functions to get the server running.

fn env_get_dimension() -> usize {
    let not_int = "variable 'OASYSDB_DIMENSION' must be an integer";
    get_env("OASYSDB_DIMENSION").parse::<usize>().expect(not_int)
}

fn get_env(key: &str) -> String {
    let not_set = format!("env variable '{}' required", key);
    env::var(key).expect(&not_set)
}
