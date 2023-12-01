pub use http::Response;
use std::collections::HashMap;

// This type will be used to serialize the generic response body.
// Example: {"status": "ok"}
pub type ResponseBody = HashMap<&'static str, &'static str>;

pub fn create_response(code: u16, body: Option<ResponseBody>) -> Response<String> {
    // Check MDN for a list of status codes.
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
    let code = http::StatusCode::from_u16(code).unwrap();

    // Serialize the body if provided.
    let _body = if !body.is_none() {
        serde_json::to_string(&body.unwrap()).unwrap()
    } else {
        // Default to an empty object.
        "{}".to_string()
    };

    // Return the response.
    Response::builder().status(code).body(_body).unwrap()
}

pub fn get_not_allowed_response() -> Response<String> {
    create_response(405, None)
}

pub fn get_not_found_response(body: Option<ResponseBody>) -> Response<String> {
    create_response(404, body)
}
