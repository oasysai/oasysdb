use crate::db::database::*;
use crate::get_env;
use rocket::http::Status;
use rocket::request::*;
use rocket::Request;
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
#[derive(Responder, Debug)]
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

pub struct Auth {
    pub token: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = &'static str;

    async fn from_request(
        request: &'r Request<'_>,
    ) -> Outcome<Self, Self::Error> {
        let server_token = get_env("OASYSDB_TOKEN");
        let token = request.headers().get_one("x-oasysdb-token");

        if token.is_none() || token.unwrap() != server_token {
            return Outcome::Error((
                Status::Unauthorized,
                "Invalid x-oasysdb-token header.",
            ));
        }

        let token = token.unwrap().to_string();
        Outcome::Success(Auth { token })
    }
}
