use super::*;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;

#[get("/values/<key>")]
pub fn get_value(db: &State<Database>, key: &str) -> (Status, Response) {
    let result = db.get_value(key);

    if result.is_ok() {
        let value = result.ok().unwrap();
        let body = serde_json::to_string(&value).unwrap();
        return (Status::Ok, Response(body));
    }

    let error = {
        let message = result.err().unwrap().0;
        let map = HashMap::from([("message", message)]);
        serde_json::to_string(&map).unwrap()
    };

    (Status::BadRequest, Response(error))
}

#[post("/values/<key>", data = "<value>")]
pub fn set_value(
    db: &State<Database>,
    key: &str,
    value: Json<Value>,
) -> (Status, Json<StringMap>) {
    let result = db.set_value(key.into(), value.into_inner());

    if result.is_ok() {
        return (Status::Ok, Json(HashMap::new()));
    }

    let message = result.err().unwrap().0;
    (Status::BadRequest, Json(HashMap::from([("message", message)])))
}
