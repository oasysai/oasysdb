pub use serde_json::Value as RequestBody;
use std::collections::HashMap;

// Headers will be parsed from the stream as a string hashmap.
// Example: "content-type": "application/json".
pub type RequestHeaders = HashMap<String, String>;

// This is the data structure that will be parsed
// from the stream and is passed to the route handlers.
pub struct Request {
    pub method: String,
    pub route: String,
    pub headers: RequestHeaders,
    pub body: RequestBody,
}
