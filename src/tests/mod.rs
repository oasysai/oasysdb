mod test_collection;
mod test_database;
mod test_distance;

use crate::prelude::*;
use rayon::iter::*;
use std::collections::HashMap;

const DIMENSION: usize = 128;
const LEN: usize = 100;

/// The test database initial collection name.
const NAME: &str = "vectors";

fn create_test_database(path: &str) -> Database {
    let mut db = Database::new(path).unwrap();
    let collection = create_collection();
    db.save_collection(NAME, &collection).unwrap();
    db
}

fn create_collection() -> Collection {
    let records = Record::many_random(DIMENSION, LEN);
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}
