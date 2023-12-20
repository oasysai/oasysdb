use super::StringMap;
use rocket::serde::json::Json;
use std::collections::HashMap;

#[get("/")]
pub fn get_status() -> Json<StringMap> {
    Json(HashMap::from([("status", "ok")]))
}

#[get("/version")]
pub fn get_version() -> Json<StringMap> {
    let version = env!("CARGO_PKG_VERSION");
    Json(HashMap::from([("version", version)]))
}
