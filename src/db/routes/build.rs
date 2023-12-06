use crate::db::server as db;
use crate::db::utils::request as req;
use crate::db::utils::response as res;
use std::collections::HashMap;

pub fn handler(
    server: &mut db::Server,
    request: &req::Request,
) -> res::Response<String> {
    match request.method.as_str() {
        "post" => post(server),
        _ => res::get_405_response(),
    }
}

fn post(server: &mut db::Server) -> res::Response<String> {
    // Build the index.
    let result = server.build();

    // If result is Err, return 500 with error message.
    if result.is_err() {
        let message = result.err().unwrap();
        return res::get_error_response(500, message);
    }

    let mut map = HashMap::new();
    map.insert("status", "success");
    let body = serde_json::to_string(&map).unwrap();
    res::create_response(200, Some(body))
}
