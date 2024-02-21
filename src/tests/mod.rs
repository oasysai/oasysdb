mod test_collection;
mod test_database;

use crate::collection::*;
use crate::database::*;
use crate::vector::*;
use rand::random;
use rayon::iter::*;
use std::collections::HashMap;

fn create_test_database(path: &str) -> Database {
    let mut db = Database::new(path).unwrap();
    let records = gen_records(128, 100);
    let records = Some(records.as_slice());
    db.create_collection("vectors", None, records).unwrap();
    db
}

fn create_collection(records: &[Record]) -> Collection {
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}

fn gen_records(dimension: usize, len: usize) -> Vec<Record> {
    let mut records = Vec::with_capacity(len);

    for _ in 0..len {
        let vector = gen_vector(dimension);
        let data = random::<usize>();
        records.push(Record::new(&vector, &data.into()));
    }

    records
}

fn gen_vector(dimension: usize) -> Vector {
    let mut vec = vec![0.0; dimension];

    for float in vec.iter_mut() {
        *float = random::<f32>();
    }

    vec.into()
}
