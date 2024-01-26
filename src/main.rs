use oasysdb::collection::*;
use oasysdb::vector::*;
use rand::random;

fn main() {
    let records = gen_records::<128>(1000);
    let records = records.as_slice();
    let query = gen_vector();
    let n = 1;

    // Compare the results.
    real_nn(records, &query, n);
    collection_built_nn(records, &query, n);
    collection_insert_nn(records, &query, n);
}

/// Finds the nearest neighbors using brute force.
/// * `records`: Vector records to search.
/// * `query`: Vector to search for.
/// * `n`: Number of nearest neighbors to find.
fn real_nn<const N: usize>(
    records: &[Record<usize, N>],
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

/// Finds the nearest neighbors using a built collection.
/// * `records`: Vector records to build the collection.
/// * `query`: Vector to search for.
/// * `n`: Number of nearest neighbors to find.
fn collection_built_nn<const N: usize>(
    records: &[Record<usize, N>],
    query: &Vector<N>,
    n: usize,
) -> Vec<(f32, usize)> {
    // Create the collection using the builder.
    let config = Config::default();
    let mut collection: Collection<usize, N> =
        Collection::build(&config, records);

    collection.delete(&VectorID(0));

    // Search the collection.
    let start = std::time::Instant::now();
    let result = collection.search(&query, n);

    print!("Collection (Built) Nearest: {}", result[0].distance);
    println!(" {:?}μs", start.elapsed().as_micros());

    result.iter().map(|c| (c.distance, c.id as usize)).collect()
}

/// Finds the nearest neighbors using a collection with inserts.
/// * `records`: Vector records to insert into the collection.
/// * `query`: Vector to search for.
/// * `n`: Number of nearest neighbors to find.
fn collection_insert_nn<const N: usize>(
    records: &[Record<usize, N>],
    query: &Vector<N>,
    n: usize,
) -> Vec<(f32, usize)> {
    // Create a new collection.
    let config = Config::default();
    let mut collection: Collection<usize, N> = Collection::new(&config);

    // Insert records into the collection.
    for record in records {
        collection.insert(record);
    }

    collection.delete(&VectorID(0));
    collection.insert(&Record { vector: gen_vector(), data: 0 });

    // Search the collection.
    let start = std::time::Instant::now();
    let result = collection.search(&query, n);

    print!("Collection (Insert) Nearest: {}", result[0].distance);
    println!(" {:?}μs", start.elapsed().as_micros());

    result.iter().map(|c| (c.distance, c.id as usize)).collect()
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
