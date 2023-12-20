use super::*;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

#[get("/values/<key>")]
pub fn get_value(db: &State<Database>, key: &str) -> (Status, Response) {
    match db.get_value(key) {
        Ok(value) => (Status::Ok, Response::from(value)),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[post("/values/<key>", data = "<value>")]
pub fn set_value(
    db: &State<Database>,
    key: &str,
    value: Json<Value>,
) -> (Status, Response) {
    match db.set_value(key.into(), value.into_inner()) {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[delete("/values/<key>")]
pub fn delete_value(db: &State<Database>, key: &str) -> (Status, Response) {
    match db.delete_value(key) {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}
