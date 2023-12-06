use crate::db::server as db;
use crate::db::utils::request as req;
use crate::db::utils::response as res;

pub fn handler(
    server: &mut db::Server,
    request: &req::Request,
) -> res::Response<String> {
    match request.method.as_str() {
        "post" => post(server, request.body.clone()),
        _ => res::get_405_response(),
    }
}

fn post(
    server: &mut db::Server,
    body: req::RequestBody,
) -> res::Response<String> {
    // Validate that embedding is in the body.
    if body.get("embedding").is_none() {
        let message = "Embedding is required.";
        return res::get_error_response(400, message);
    }

    // Get the embedding from the request body.
    let embedding: Vec<f32> =
        match serde_json::from_value(body["embedding"].clone()) {
            Ok(vec) => vec,
            Err(_) => {
                let m = "Embedding must be an array of floats.";
                return res::get_error_response(400, m);
            }
        };

    // Get optional count from the request body.
    let count: u16 = match body["count"].as_u64() {
        Some(v) => v as u16,
        None => 5,
    };

    // Search the nearest neighbors.
    let result = server.search(embedding, count.into());

    // If result is Err, return 500 with error message.
    if result.is_err() {
        let message = result.err().unwrap();
        return res::get_error_response(400, message);
    }

    // Serialize the result as a string for the response.
    let body = {
        let _val: Vec<db::Data> = result.unwrap();
        serde_json::to_string(&_val).unwrap()
    };

    res::create_response(200, Some(body))
}
