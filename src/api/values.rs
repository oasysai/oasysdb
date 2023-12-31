// Endpoints will be prefixed with /values.

use super::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

#[get("/<key>")]
pub fn get_value(
    db: &State<Database>,
    key: &str,
    _auth: Auth,
) -> (Status, Response) {
    match db.get_value(key) {
        Ok(value) => (Status::Ok, Response::from(value)),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[post("/<key>", data = "<value>")]
pub fn set_value(
    db: &State<Database>,
    key: &str,
    value: Json<Value>,
    _auth: Auth,
) -> (Status, Response) {
    match db.set_value(key, value.into_inner()) {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[delete("/<key>")]
pub fn delete_value(
    db: &State<Database>,
    key: &str,
    _auth: Auth,
) -> (Status, Response) {
    match db.delete_value(key) {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[delete("/")]
pub fn reset_values(db: &State<Database>, _auth: Auth) -> (Status, Response) {
    match db.reset_values() {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}
