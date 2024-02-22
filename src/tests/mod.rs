mod test_collection;
mod test_database;

use crate::collection::*;
use crate::database::*;
use crate::vector::*;
use rayon::iter::*;
use std::collections::HashMap;

fn create_test_database(path: &str) -> Database {
    let mut db = Database::new(path).unwrap();
    let records = Record::many_random(128, 100);
    let records = Some(records.as_slice());
    db.create_collection("vectors", None, records).unwrap();
    db
}

fn create_collection(records: &[Record]) -> Collection {
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}
