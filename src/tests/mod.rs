use crate::api::*;
use crate::create_server;
use crate::db::database::*;
use rand::distributions::*;
use rand::*;
use rocket::http::Status;
use rocket::local::blocking::Client;
use std::collections::HashMap;

mod test_utils;

fn create_test_client() -> Client {
    let path = format!("data/tests/{}", random_string(10));
    let config = Config { path, dimension: 2 };
    let db = Database::new(config);

    // Prepopulate database with random values.
    for i in 0..9 {
        let embedding = vec![random::<f32>(); 2];
        let value = Value { embedding, data: HashMap::new() };
        db.set_value(&i.to_string(), value).unwrap();
    }

    // Build initial graph for testing.
    let _ = {
        let name = "default".to_string();
        let config = GraphConfig { name, ef_construction: 10, ef_search: 10 };
        db.create_graph(config)
    };

    let rocket = create_server(db);
    Client::tracked(rocket).unwrap()
}

fn random_string(length: usize) -> String {
    Alphanumeric.sample_string(&mut thread_rng(), length)
}
