use oasysdb::api::*;
use oasysdb::db::database::*;
use oasysdb::*;

// Other imports.
use dotenv::dotenv;
use rocket::http::Status;

#[macro_use]
extern crate rocket;

#[catch(404)]
fn catch_404() -> (Status, Response) {
    let message = "Invalid endpoint or method.";
    (Status::NotFound, Response::error(message))
}

#[catch(401)]
fn catch_401() -> (Status, Response) {
    let message = "Invalid x-oasysdb-token header.";
    (Status::Unauthorized, Response::error(message))
}

#[launch]
fn rocket() -> _ {
    // Load environment variables from .env file.
    // This is only needed for development.
    dotenv().ok();

    // Get environment variables.
    let _token = get_env("OASYSDB_TOKEN");
    let dimension = env_get_dimension();

    // Create database configuration.
    let config = {
        let path = "data".to_string();
        Config { path, dimension }
    };

    // Initialize shared database state.
    let db = Database::new(config);

    // Display log.
    println!("OasysDB is running on port 3141.");
    println!("OasysDB accepts embeddings of {} dimension.", dimension);

    rocket::build()
        .manage(db)
        .mount("/", routes![get_status, get_version])
        .mount("/values", routes![set_value, get_value, delete_value])
        .mount("/graphs", routes![create_graph, delete_graph, query_graph])
        .register("/", catchers![catch_401, catch_404])
}

fn env_get_dimension() -> usize {
    let not_int = "variable 'OASYSDB_DIMENSION' must be an integer";
    get_env("OASYSDB_DIMENSION").parse::<usize>().expect(not_int)
}
