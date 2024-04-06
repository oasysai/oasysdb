use oasysdb::prelude::*;

/// Creates a collection with random vector records.
/// * `dimension`: Dimensionality of the vector embeddings
/// * `len`: Number of records in the database
pub fn build_test_collection(dimension: usize, len: usize) -> Collection {
    let records = Record::many_random(dimension, len);
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}
