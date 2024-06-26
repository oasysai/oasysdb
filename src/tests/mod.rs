use crate::db::*;
use crate::types::*;
use arrow::datatypes::{DataType, Field};
use std::fs;
use std::path::PathBuf;

mod test_collection;
mod test_database;

const TEST_DIR: &str = "odb_data";
const TEST_COLLECTION: &str = "collection";

fn create_test_database() -> Result<Database, Error> {
    // Reset the database directory for testing.
    let path = PathBuf::from(TEST_DIR);
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }

    // The database should have some subdirectories.
    let db = Database::open(path.clone())?;
    let content = path.read_dir()?;
    assert!(content.count() == 2);

    // Create a test collection.
    db._create_collection(TEST_COLLECTION)?;

    Ok(db)
}

fn get_test_collection() -> Result<Collection, Error> {
    let db = create_test_database()?;
    let collection_refs = db.state()?.collection_refs;

    let directory = collection_refs[TEST_COLLECTION].to_path_buf();
    let collection = Collection::open(directory)?;
    Ok(collection)
}
