pub mod client;
pub mod server;

use serde_json::to_string;
use std::collections::HashMap;

pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> ErrorResponse {
        let code = code.to_string();
        let message = message.to_string();
        ErrorResponse { code, message }
    }

    pub fn response(&self) -> String {
        let mut map = HashMap::new();
        map.insert("status", self.code.clone());
        map.insert("message", self.message.clone());
        to_string(&map).unwrap()
    }
}
