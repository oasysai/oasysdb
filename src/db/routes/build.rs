use crate::db::server as db;
use crate::db::utils::request::{Request, RequestBody};
use crate::db::utils::response as res;
use std::collections::HashMap;

pub fn handler(
    server: &mut db::Server,
    request: &Request,
) -> res::Response<String> {
    match request.method.as_str() {
        "post" => post(server, request.body.clone()),
        _ => res::get_405_response(),
    }
}

fn post(server: &mut db::Server, body: RequestBody) -> res::Response<String> {
    // Get optional build parameters from the body.
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

    // Build the index.
    let result = server.build(ef_search, ef_construction);

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
