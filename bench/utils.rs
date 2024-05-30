use oasysdb::prelude::*;

/// Creates a collection with random vector records.
/// * `dimension`: Dimensionality of the vector embeddings
/// * `len`: Number of records in the database
pub fn build_test_collection(dimension: usize, len: usize) -> Collection {
    let records = Record::many_random(dimension, len);
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}

/// Creates a pre-populated database with a collection for testing.
/// * `dimension`: Dimensionality of the vector embeddings
/// * `size`: Number of records in the collection
pub fn create_test_database(dimension: usize, size: usize) -> Database {
    let collection = build_test_collection(dimension, size);
    let mut db = Database::new("data/bench").unwrap();
    db.save_collection("bench", &collection).unwrap();
    db
}
