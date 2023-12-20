use crate::db::server::{Data, Server};
use crate::db::utils::request::{Request, RequestBody};
use crate::db::utils::response as res;
use std::collections::HashMap;

pub fn handler(server: &Server, request: &Request) -> res::Response<String> {
    // Graph-related routes only accept POST requests.
    if request.method != "post" {
        return res::get_405_response();
    }

    // Match the exact route to determine if the server
    // should build the graph or query it.
    match request.route.as_str() {
        "/graphs" => post_graphs(server, request.body.clone()),
        "/graphs/query" => post_graphs_query(server, request.body.clone()),
        _ => res::get_404_response(),
    }
}

fn post_graphs(server: &Server, body: RequestBody) -> res::Response<String> {
    // The name of the graph to be built.
    let name = match body["name"].as_str() {
        Some(name) => name,
        None => "default",
    };

    // EF search is maximum number of candidate neighbors
    // to be considered during search.
    let ef_search = match body["ef_search"].as_u64() {
        Some(int) => int as usize,
        None => 100,
    };

    // EF construction is the maximum number of candidate
    // neighbors to consider when connecting a newly inserted
    // point to the existing graph.
    let ef_construction = match body["ef_construction"].as_u64() {
        Some(int) => int as usize,
        None => 100,
    };

    // Build the graph.
    let result = server.build(name.into(), ef_search, ef_construction);

    // If result is Err, return 500 with error message.
    if result.is_err() {
        let message = result.err().unwrap();
        return res::get_error_response(500, message);
    }

    // Create and return a simple success response.
    let mut map = HashMap::new();
    map.insert("status", "success");
    let body = serde_json::to_string(&map).unwrap();
    res::create_response(200, Some(body))
}

fn post_graphs_query(
    server: &Server,
    body: RequestBody,
) -> res::Response<String> {
    // Validate that embedding is in the body.
    if body.get("embedding").is_none() {
        let message = "Embedding is required.";
        return res::get_error_response(400, message);
    }

    // Get the embedding from the request body.
    let embedding: Vec<f32> =
        match serde_json::from_value(body["embedding"].clone()) {
            Ok(vec) => vec,
            Err(_) => {
                let m = "Embedding must be an array of floats.";
                return res::get_error_response(400, m);
            }
        };

    // Get optional graph name.
    let name = match body["name"].as_str() {
        Some(name) => name,
        None => "default",
    };

    // Get optional count from the request body.
    let count: u16 = match body["count"].as_u64() {
        Some(v) => v as u16,
        None => 5,
    };

    // query the nearest neighbors.
    let result = server.query(name.into(), embedding, count.into());

    // If result is Err, return 500 with error message.
    if result.is_err() {
        let message = result.err().unwrap();
        return res::get_error_response(400, message);
    }

    // Serialize the result as a string for the response.
    let body = {
        let _val: Vec<Data> = result.unwrap();
        serde_json::to_string(&_val).unwrap()
    };

    res::create_response(200, Some(body))
}
