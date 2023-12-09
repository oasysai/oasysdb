use crate::db::server::{Server, Value};
use crate::db::utils::request::{Request, RequestBody};
use crate::db::utils::response as res;

pub fn handler(
    server: &mut Server,
    request: &Request,
) -> res::Response<String> {
    let route = request.route.clone();
    let body = request.body.clone();
    match request.method.as_str() {
        "get" => get(server, route),
        "post" => post(server, body),
        "delete" => delete(server, route),
        _ => res::get_405_response(),
    }
}

fn get(server: &Server, route: String) -> res::Response<String> {
    // Get the key from the route.
    let route_parts: Vec<&str> = route.split('/').collect();
    let key = route_parts.last().unwrap().to_string();

    // If key is empty, return 400 with error message.
    if key.is_empty() || route_parts.len() < 3 {
        let message = "The key is required.";
        return res::get_error_response(400, message);
    }

    // Get the value from the key-value store.
    let value = server.get(key.clone());

    // If value is None, return 404 with error message.
    if value.is_err() {
        let message = value.err().unwrap();
        return res::get_error_response(404, message);
    }

    // Serialize value as string for the response.
    let body = {
        let _val: Value = value.unwrap();
        serde_json::to_string(&_val).unwrap()
    };

    res::create_response(200, Some(body))
}

fn post(server: &mut Server, body: RequestBody) -> res::Response<String> {
    // If request body is missing key or value.
    if body.get("key").is_none() || body.get("value").is_none() {
        let message = "Both key and value are required.";
        return res::get_error_response(400, message);
    }

    // Get the key from request body.
    // Validate that key is string.
    let key: String = match body["key"].as_str() {
        Some(key) => key.to_string(),
        None => {
            let message = "The key must be a string.";
            return res::get_error_response(400, message);
        }
    };

    // Get the value from request body.
    // Validate that value is a Value struct.
    let _val = body["value"].clone();
    let value: Value = match serde_json::from_value(_val) {
        Ok(value) => value,
        Err(_) => {
            let message = "The value provided is invalid.";
            return res::get_error_response(400, message);
        }
    };

    // Insert the key-value pair into the key-value store.
    let result = server.set(key, value);

    // If result is Err, return 400 with error message.
    if result.is_err() {
        let message = result.err().unwrap();
        return res::get_error_response(400, message);
    }

    // Serialize value as string for the response.
    let body = {
        let _val = result.unwrap();
        serde_json::to_string(&_val).unwrap()
    };

    res::create_response(201, Some(body))
}

fn delete(server: &Server, route: String) -> res::Response<String> {
    // Get the key from the route.
    let route_parts: Vec<&str> = route.split('/').collect();
    let key = route_parts.last().unwrap().to_string();

    // If key is empty, return 400 with error message.
    if key.is_empty() || route_parts.len() < 3 {
        let message = "The key is missing.";
        return res::get_error_response(400, message);
    }

    // Delete the key-value pair from the store.
    let result = server.delete(key.clone());

    // Handle error when deleting key-value pair.
    if result.is_err() {
        let message = result.err().unwrap();
        return res::get_error_response(400, message);
    }

    // Return empty success response.
    res::create_response(204, None)
}
