use oasysdb::index::*;
use oasysdb::vector::*;
use rand::random;

fn main() {
    let records = gen_records::<128>(1000);
    let records = records.as_slice();
    let query = gen_vector();
    let n = 1;

    real_nn(records, &query, n);
    index_built_nn(records, &query, n);
    index_insert_nn(records, &query, n);
}

fn real_nn<const N: usize>(
    records: &[IndexRecord<usize, N>],
    query: &Vector<N>,
    n: usize,
) -> Vec<(f32, usize)> {
    let start = std::time::Instant::now();

    let mut nearest = Vec::with_capacity(records.len());

    // Calculate the distance between the query and each record.
    for record in records {
        let distance = query.distance(&record.vector);
        nearest.push((distance, record.data));
    }

    // Sort the nearest neighbors by distance.
    nearest.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    nearest.truncate(n);

    print!("Real Nearest: {:?}", nearest[0].0);
    println!(" {:?}μs", start.elapsed().as_micros());

    nearest
}

fn index_built_nn<const N: usize>(
    records: &[IndexRecord<usize, N>],
    query: &Vector<N>,
    n: usize,
) -> Vec<(f32, usize)> {
    // Build the index.
    let config = IndexConfig::default();
    let mut hnsw: IndexGraph<usize, N> = IndexGraph::build(&config, records);

    hnsw.delete(&VectorID(0));

    // Query the index.
    let start = std::time::Instant::now();
    let result = hnsw.search(&query, n);

    print!("Index (Built) Nearest: {}", result[0].distance);
    println!(" {:?}μs", start.elapsed().as_micros());

    result.iter().map(|c| (c.distance, c.id as usize)).collect()
}

fn index_insert_nn<const N: usize>(
    records: &[IndexRecord<usize, N>],
    query: &Vector<N>,
    n: usize,
) -> Vec<(f32, usize)> {
    // Build the index.
    let config = IndexConfig::default();
    let mut hnsw: IndexGraph<usize, N> = IndexGraph::new(&config);

    // Insert records into the index.
    for record in records {
        hnsw.insert(record);
    }

    hnsw.delete(&VectorID(0));

    // Query the index.
    let start = std::time::Instant::now();
    let result = hnsw.search(&query, n);

    print!("Index (Insert) Nearest: {}", result[0].distance);
    println!(" {:?}μs", start.elapsed().as_micros());

    result.iter().map(|c| (c.distance, c.id as usize)).collect()
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
