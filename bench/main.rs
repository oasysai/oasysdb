// See measure-memory.rs example for memory usage.

mod utils;

use criterion::*;
use oasysdb::prelude::*;
use utils::*;

/// The number of vector records in the collection.
const COLLECTION_SIZE: usize = 1_000_000;

/// The vector embedding dimension.
/// A vector dimension of 768, 1024, or 4096 are very common options
/// for models on [MTEB](https://huggingface.co/spaces/mteb/leaderboard).
const DIMENSION: usize = 128;

fn bench_search_collection(criterion: &mut Criterion) {
    let id = "search collection";

    // Create the collection.
    let collection = build_test_collection(DIMENSION, COLLECTION_SIZE);

    // Create a random vector to search for.
    let vector = Vector::random(DIMENSION);

    // Benchmark the search speed.
    let routine = || {
        black_box(collection.search(&vector, 10).unwrap());
    };

    criterion.bench_function(id, |b| b.iter(routine));
}

fn bench_true_search_collection(criterion: &mut Criterion) {
    let id = "true search collection";

    // Create the collection.
    let collection = build_test_collection(DIMENSION, COLLECTION_SIZE);

    // Create a random vector to search for.
    let vector = Vector::random(DIMENSION);

    // Benchmark the search speed.
    let routine = || {
        black_box(collection.true_search(&vector, 10).unwrap());
    };

    criterion.bench_function(id, |b| b.iter(routine));
}

fn bench_insert_to_collection(criterion: &mut Criterion) {
    let id = "insert to collection";

    // Create the initial collection.
    let mut collection = build_test_collection(DIMENSION, COLLECTION_SIZE);

    // Benchmark the insert speed.
    let record = Record::random(DIMENSION);
    criterion.bench_function(id, |bencher| {
        bencher.iter(|| {
            black_box(collection.insert(&record).unwrap());
        })
    });
}

criterion_group!(
    collection,
    bench_search_collection,
    bench_true_search_collection,
    bench_insert_to_collection
);

fn bench_save_collection_to_database(criterion: &mut Criterion) {
    let id = "save collection to database";

    // Setup the database and collection.
    let collection = build_test_collection(DIMENSION, COLLECTION_SIZE);
    let mut db = create_test_database(DIMENSION, COLLECTION_SIZE);

    // Benchmark the save speed.
    criterion.bench_function(id, |bencher| {
        bencher.iter(|| {
            black_box(db.save_collection("bench", &collection).unwrap());
        })
    });
}

fn bench_get_collection_from_database(criterion: &mut Criterion) {
    let id = "get collection from database";
    let db = create_test_database(DIMENSION, COLLECTION_SIZE);

    // Benchmark the get speed.
    // This is the operation that loads the collection into memory.
    let routine = || {
        black_box(db.get_collection("bench").unwrap());
    };

    criterion.bench_function(id, |b| b.iter(routine));
}

criterion_group! {
    name = database;
    config = Criterion::default().sample_size(10);
    targets =
        bench_save_collection_to_database,
        bench_get_collection_from_database
}

criterion_main!(collection, database);
