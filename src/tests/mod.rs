mod test_collection;
mod test_database;
mod test_distance;
mod test_metadata;

use crate::prelude::*;
use rayon::iter::*;
use std::collections::HashMap;

const DIMENSION: usize = 128;
const LEN: usize = 100;

/// The test database initial collection name.
const NAME: &str = "vectors";

fn create_test_database() -> Database {
    let mut db = Database::new("data/rs").unwrap();
    let collection = create_collection();
    db.save_collection(NAME, &collection).unwrap();
    db
}

fn create_collection() -> Collection {
    let all_records = Record::many_random(DIMENSION, LEN);

    // Split the records into two halves.
    // The first half is used to build the collection.
    // The second half is used to insert.
    let mid = LEN / 2;
    let first_half = &all_records[0..mid];
    let second_half = &all_records[mid..LEN];

    let config = Config::default();
    let mut collection = Collection::build(&config, first_half).unwrap();

    collection.insert_many(second_half).unwrap();
    collection
}
