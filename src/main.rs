use oasysdb::index::*;
use oasysdb::vector::Vector;
use rand::random;

fn main() {
    let config = IndexConfig::default();
    let records = gen_records::<128>(1000);
    let records = records.as_slice();
    let hnsw: IndexGraph<usize, 128> = IndexGraph::build(&config, records);
    let query = gen_vector();

    // Search the index for the nearest neighbors.
    let start = std::time::Instant::now();
    let hnsw_nearest = hnsw.search(&query, 1);
    print!("Index Nearest: {}", hnsw_nearest[0].distance);
    println!(" {:?}μs", start.elapsed().as_micros());

    // Calculate the real nearest neighbors using brute force.
    let start = std::time::Instant::now();
    let real_nearest = real_nearest_neighbors(records, &query, 1);
    print!("Real Nearest: {:?}", real_nearest[0].0);
    println!(" {:?}μs", start.elapsed().as_micros());
}

fn real_nearest_neighbors<const N: usize>(
    records: &[IndexRecord<usize, N>],
    query: &Vector<N>,
    n: usize,
) -> Vec<(f32, usize)> {
    let mut nearest = Vec::with_capacity(records.len());

    for record in records {
        let distance = query.distance(&record.vector);
        nearest.push((distance, record.data));
    }

    nearest.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    nearest.truncate(n);
    nearest
}

fn gen_records<const N: usize>(len: usize) -> Vec<IndexRecord<usize, N>> {
    let mut records = Vec::with_capacity(len);

    for _ in 0..len {
        let vector = gen_vector::<N>();
        let data = random::<usize>();
        records.push(IndexRecord { vector, data });
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
