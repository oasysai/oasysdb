use dotenv::dotenv;
use oasysdb::db::server::{Config, Server};
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
    let config = Config { dimension };

    // Create and start the server.
    let host = "0.0.0.0";
    let mut server = Server::new(host, port.as_str(), config);
    server.serve().await;

    // Display the server configuration.
    println!("OasysDB is running on port {}.", port);
    println!("OasysDB accepts embeddings of {} dimension.", dimension);
}

fn env_get_dimension() -> usize {
    let not_set = "env variable 'OASYSDB_DIMENSION' required";
    let not_int = "variable 'OASYSDB_DIMENSION' must be an integer";
    env::var("OASYSDB_DIMENSION")
        .expect(not_set)
        .parse::<usize>()
        .expect(not_int)
}
