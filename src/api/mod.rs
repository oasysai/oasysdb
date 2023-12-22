use crate::db::database::*;
use crate::get_env;
use rocket::http::Status;
use rocket::request::*;
use rocket::Request;
use serde::Serialize;
use std::collections::HashMap;

// Initialize modules.
mod graphs;
mod utils;
mod values;

// Export all of the endpoints from the modules.
pub use graphs::*;
pub use utils::*;
pub use values::*;

/// A **generic body** type that can be used by all endpoints to
/// return any response. The string will be converted to JSON by
/// Rocket automatically.
///
/// This is not the recommended way to return responses in Rocket
/// compared to using `Json` and that's why if possible, use
/// the implementations to minimize errors.
#[derive(Responder, Debug)]
#[response(content_type = "json")]
pub struct Response(String);

impl Response {
    /// Creates an empty object.
    pub fn empty() -> Response {
        Response(String::from("{}"))
    }

    /// Creates a standard error object like this:
    /// ```json
    /// { "error": "message" }
    /// ```
    pub fn error(message: &str) -> Response {
        let map = HashMap::from([("error", message)]);
        let body = serde_json::to_string(&map).unwrap();
        Response(body)
    }

    /// Creates an object from a custom data type. This requires the
    /// data to have derived from the `Serialize` trait.
    ///
    /// # Example
    ///
    /// ```rs
    /// #[derive(Serialize)]
    /// struct Data {}
    /// let response = Response::from(Data {});
    /// ```
    pub fn from<Value: Serialize>(value: Value) -> Response {
        let body = serde_json::to_string(&value).unwrap();
        Response(body)
    }
}

/// A custom data type that is used to authenticate requests.
/// When handling routes that are private, we can add this type to
/// the function parameters and Rocket will automatically check if
/// the request has the correct token.
///
/// # Example
///
/// ```rs
/// #[get("/private")]
/// pub fn private_route(_auth: Auth) {}
/// ```
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
