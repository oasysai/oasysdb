use oasysdb::api::*;
use oasysdb::db::database::*;
use oasysdb::*;

// Other imports.
use dotenv::dotenv;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    // Load environment variables from .env file.
    // This is only needed for development.
    dotenv().ok();

    // Get environment variables.
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
        .mount("/", routes![get_status])
        .mount("/", routes![get_version])
        .mount("/", routes![set_value])
        .mount("/", routes![get_value])
}

fn env_get_dimension() -> usize {
    let not_int = "variable 'OASYSDB_DIMENSION' must be an integer";
    get_env("OASYSDB_DIMENSION").parse::<usize>().expect(not_int)
}
