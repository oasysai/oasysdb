use super::utils::response as res;
use super::utils::stream;
use crate::db::server::Server;
use tokio::net::TcpStream;

mod graphs;
mod root;
mod values;
mod version;

// In this module, we define the route handlers
// for the database server HTTP API.
//
// File format: <path>.rs
// Example: version.rs of /version.
//
// Inside the file, we define the public "handler"
// function which takes a reference to a Request
// and returns a Response.
//
// Inside the handler function, we define the functions
// that handle different HTTP methods.
// Function name format: <method>
// Example: get, post, put, delete, etc.
//
// Note: Avoid wildcard imports.

pub async fn handle_request(server: &Server, stream: &mut TcpStream) {
    // Read request from the client.
    let _req = stream::read(stream).await;

    // Handle disconnection or invalid request.
    // Return invalid request response.
    if _req.is_none() {
        let response = res::get_error_response(400, "Invalid request.");
        stream::write(stream, response).await;
        return;
    }

    // Unwrap the data.
    let request = _req.as_ref().unwrap();
    let route = request.route.clone();

    // Check if the route is private.
    // Private routes require authentication.
    let private_routes = ["/graphs", "/values"];
    if private_routes.iter().any(|r| route.starts_with(r)) {
        // Get the token from the request headers.
        let token = request.headers.get("x-oasysdb-token");

        // Check if the token is valid.
        // If not, return unauthorized response.
        if token.is_none() || token.unwrap() != &server.config.token {
            let response = res::get_401_response();
            stream::write(stream, response).await;
            return;
        }
    }

    // Get response based on different routes and methods.
    let response = match route.as_str() {
        "/" => root::handler(request),
        "/version" => version::handler(request),
        _ if route.starts_with("/graphs") => graphs::handler(server, request),
        _ if route.starts_with("/values") => values::handler(server, request),
        _ => res::get_404_response(),
    };

    // Write the data back to the client.
    stream::write(stream, response).await;
}
