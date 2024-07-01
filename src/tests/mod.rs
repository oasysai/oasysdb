use crate::db::*;
use crate::types::*;
use arrow::array::{self, Array};
use arrow::datatypes::{DataType, Field};
use rand::random;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

mod stress_test_database;
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

fn create_test_database_with_data() -> Result<Database, Error> {
    let db = create_test_database()?;
    populate_database(db)
}

fn generate_random_vectors(dimension: usize, len: usize) -> Vec<Vec<f32>> {
    (0..len)
        .map(|_| (0..dimension).map(|_| random::<f32>()).collect())
        .collect()
}

fn populate_database(database: Database) -> Result<Database, Error> {
    let fields = ["vector", "title", "year"];
    let field_names: Vec<String> =
        fields.iter().map(|f| f.to_string()).collect();

    let vectors = generate_random_vectors(128, 3);
    let titles = vec!["The Matrix", "Avatar", "Inception"];
    let years = vec![1999, 2009, 2010];

    let records = vec![
        Arc::new(array::ListArray::from_vectors(vectors)) as Arc<dyn Array>,
        Arc::new(array::StringArray::from(titles)) as Arc<dyn Array>,
        Arc::new(array::Int32Array::from(years)) as Arc<dyn Array>,
    ];

    database._insert_records(TEST_COLLECTION, &field_names, &records)?;
    Ok(database)
}
