mod test_database;
mod test_index;

use crate::collection::*;
use crate::vector::*;
use rand::random;

fn create_collection<const N: usize>(
    records: &[Record<usize, N>],
) -> Collection<usize, N> {
    let config = Config::default();
    Collection::build(&config, &records)
}

fn gen_records<const N: usize>(len: usize) -> Vec<Record<usize, N>> {
    let mut records = Vec::with_capacity(len);

    for _ in 0..len {
        let vector = gen_vector::<N>();
        let data = random::<usize>();
        records.push(Record { vector, data });
    }

    records
}

fn gen_vector<const N: usize>() -> Vector<N> {
    let mut vec = [0.0; N];

    for float in vec.iter_mut() {
        *float = random::<f32>();
    }

    Vector(vec)
}

fn brute_force_search<const N: usize>(
    records: &[Record<usize, N>],
    query: &Vector<N>,
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
