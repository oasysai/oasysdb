pub use http::Response;
use std::collections::HashMap;

pub fn create_response(code: u16, body: Option<String>) -> Response<String> {
    // Check MDN for a list of status codes.
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
    let code = http::StatusCode::from_u16(code).unwrap();
    let body = body.unwrap_or(String::from("{}"));
    Response::builder().status(code).body(body).unwrap()
}

pub fn get_error_response(code: u16, message: &str) -> Response<String> {
    let mut map = HashMap::new();
    map.insert("error", message);
    let body = serde_json::to_string(&map).unwrap();
    create_response(code, Some(body))
}

// Generic empty error responses.
// This is useful for routes that don't need to return
// a body and has a canonical status code.

pub fn get_405_response() -> Response<String> {
    create_response(405, None)
}

pub fn get_404_response() -> Response<String> {
    create_response(404, None)
}
