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

criterion_group!(
    collection,
    bench_search_collection,
    bench_true_search_collection
);

criterion_main!(collection);
