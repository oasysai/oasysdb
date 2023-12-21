use super::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;

#[post("/", data = "<data>")]
pub fn create_graph(
    db: &State<Database>,
    data: Json<GraphConfig>,
) -> (Status, Response) {
    match db.create_graph(data.into_inner()) {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[delete("/<name>")]
pub fn delete_graph(db: &State<Database>, name: &str) -> (Status, Response) {
    match db.delete_graph(name) {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[derive(Deserialize)]
pub struct QueryGraphBody {
    embedding: Embedding,
    k: Option<usize>,
}

#[post("/<name>/query", data = "<data>")]
pub fn query_graph(
    db: &State<Database>,
    name: &str,
    data: Json<QueryGraphBody>,
) -> (Status, Response) {
    let data = data.into_inner();

    // Default value for k is 10.
    let k = data.k.unwrap_or(10);

    match db.query_graph(name, data.embedding, k) {
        Ok(data) => (Status::Ok, Response::from(data)),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}
