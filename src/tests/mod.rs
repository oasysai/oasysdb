mod test_collection;
mod test_database;

use crate::collection::*;
use crate::database::*;
use crate::vector::*;
use rayon::iter::*;
use std::collections::HashMap;

const DIMENSION: usize = 128;
const LEN: usize = 100;

/// The test database initial collection name.
const NAME: &str = "vectors";

fn create_test_database(path: &str) -> Database {
    let mut db = Database::new(path).unwrap();
    let records = Record::many_random(DIMENSION, LEN);
    db.create_collection("vectors", None, Some(records)).unwrap();
    db
}

fn create_collection(records: &[Record]) -> Collection {
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}
