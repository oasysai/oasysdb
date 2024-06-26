use crate::db::*;
use crate::types::*;
use arrow::datatypes::{DataType, Field};
use std::fs;
use std::path::PathBuf;

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

    // Add a couple of fields to the collection.
    let field_title = Field::new("title", DataType::Utf8, true);
    let field_year = Field::new("year", DataType::Int32, true);
    db._add_fields(TEST_COLLECTION, vec![field_title, field_year])?;

    Ok(db)
}
