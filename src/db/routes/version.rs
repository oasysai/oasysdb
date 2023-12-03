use crate::db::utils::request as req;
use crate::db::utils::response as res;
use std::collections::HashMap;

pub fn handler(request: &req::Request) -> res::Response<String> {
    match request.method.as_str() {
        "get" => get(),
        _ => res::get_405_response(),
    }
}

fn get() -> res::Response<String> {
    // Get the version from the Cargo.toml file.
    let ver = env!("CARGO_PKG_VERSION");

    // Create a HashMap to store the version.
    let mut map = HashMap::new();
    map.insert("version", ver);
    let body = serde_json::to_string(&map).unwrap();

    res::create_response(200, Some(body))
}
