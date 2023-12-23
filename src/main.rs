use oasysdb::db::database::*;
use oasysdb::*;

// Other imports.
use dotenv::dotenv;

#[macro_use]
extern crate rocket;

#[cfg(test)]
mod tests;

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

    // Display log.
    println!("OasysDB is running on port 3141.");
    println!("OasysDB accepts embeddings of {} dimension.", dimension);

    let db = Database::new(config);
    create_server(db)
}

fn env_get_dimension() -> usize {
    let not_int = "variable 'OASYSDB_DIMENSION' must be an integer";
    get_env("OASYSDB_DIMENSION").parse::<usize>().expect(not_int)
}
