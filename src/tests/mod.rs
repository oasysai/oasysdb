use crate::api::*;
use crate::create_server;
use crate::db::database::*;
use rand::random;
use rocket::http::*;
use rocket::local::blocking::Client;
use std::collections::HashMap;
use std::env;

mod test_graphs;
mod test_utils;
mod test_values;

/// Returns a valid `x-oasysdb-token` header for testing.
///
/// # Example
///
/// ```rs
/// let header = get_auth_header();
/// let response = client.get("/").header(header).dispatch();
/// ```
fn get_auth_header() -> Header<'static> {
    Header::new("x-oasysdb-token", "token")
}

/// Creates a test client with a prepopulated database and default graph
/// built. `id` is used to create a dedicated folder for the database.
/// This allows for multiple tests to run in parallel.
///
/// # Example
///
/// ```rs
/// let client = create_test_client("test_name");
/// ```
fn create_test_client(id: &str) -> Client {
    // Set environment variables for testing.
    env::set_var("OASYSDB_DIMENSION", "2");
    env::set_var("OASYSDB_TOKEN", "token");

    let path = format!("data/tests/{}", id);
    let config = Config { path, dimension: 2 };
    let db = Database::new(config);

    // Prepopulate database with random values.
    for i in 0..9 {
        let embedding = vec![random::<f32>(); 2];
        let value = Value { embedding, data: HashMap::new() };
        db.set_value(&i.to_string(), value).unwrap();
    }

    let config = GraphConfig {
        name: "default".to_string(),
        ef_construction: 10,
        ef_search: 10,
        filter: None,
    };

    // Build initial graph for testing.
    let _ = db.create_graph(config);

    let rocket = create_server(db);
    Client::tracked(rocket).unwrap()
}
