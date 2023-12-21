use crate::db::database::*;
use serde::Serialize;
use std::collections::HashMap;

mod graphs;
mod utils;
mod values;

pub use graphs::*;
pub use utils::*;
pub use values::*;

// Not the recommended way to do this as this requires manually
// serializing the response. Be careful with this approach.
#[derive(Responder)]
#[response(content_type = "json")]
pub struct Response(String);

impl Response {
    pub fn empty() -> Response {
        Response(String::from("{}"))
    }

    pub fn error(message: &str) -> Response {
        let map = HashMap::from([("error", message)]);
        let body = serde_json::to_string(&map).unwrap();
        Response(body)
    }

    pub fn from<Value: Serialize>(value: Value) -> Response {
        let body = serde_json::to_string(&value).unwrap();
        Response(body)
    }
}
