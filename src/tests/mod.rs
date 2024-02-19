mod test_collection;
mod test_database;

use crate::collection::*;
use crate::database::*;
use crate::vector::*;
use rand::random;

fn create_test_database(path: &str) -> Database {
    let mut db = Database::new(path).unwrap();
    let records = gen_records(128, 100);
    let records = Some(records.as_slice());
    db.create_collection::<usize, 32>("vectors", None, records).unwrap();
    db
}

fn create_collection(records: &[Record<usize>]) -> Collection<usize> {
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}

fn gen_records(dimension: usize, len: usize) -> Vec<Record<usize>> {
    let mut records = Vec::with_capacity(len);

    for _ in 0..len {
        let vector = gen_vector(dimension);
        let data = random::<usize>();
        records.push(Record { vector, data });
    }

    records
}

fn gen_vector(dimension: usize) -> Vector {
    let mut vec = vec![0.0; dimension];

    for float in vec.iter_mut() {
        *float = random::<f32>();
    }

    Vector(vec)
}

fn brute_force_search(
    records: &[Record<usize>],
    query: &Vector,
    n: usize,
) -> Vec<(f32, usize)> {
    let mut nearest = Vec::with_capacity(records.len());

    // Calculate the distance between the query and each record.
    for record in records {
        let distance = query.distance(&record.vector);
        nearest.push((distance, record.data));
    }

    // Sort the nearest neighbors by distance.
    nearest.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    nearest.truncate(n);
    nearest
}
