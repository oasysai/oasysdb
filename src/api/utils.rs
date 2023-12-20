use rocket::serde::json::Json;
use std::collections::HashMap;

type StringMap = HashMap<&'static str, &'static str>;
type Response = Json<StringMap>;

#[get("/")]
pub fn get_status() -> Response {
    Json(HashMap::from([("status", "ok")]))
}

#[get("/version")]
pub fn get_version() -> Response {
    let version = env!("CARGO_PKG_VERSION");
    Json(HashMap::from([("version", version)]))
}
