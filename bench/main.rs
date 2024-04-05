//! Benchmark lookup times. See `measure-memory.rs` example for memory usage.
mod utils;

use criterion::*;
use oasysdb::prelude::*;
use utils::*;


pub mod config {
    /// The number of records in the database.
    pub static DATABASE_SIZE: usize = 1_000_000;

    /// The dimensionality of the vector embeddings.
    /// 768 performs well for most [MTEB](https://huggingface.co/spaces/mteb/leaderboard) models.
    pub static DIMENSION: usize = 128;
}

fn bench_search_collection(criterion: &mut Criterion) {
    let id = "Search collection";

    let vector = Vector::random(config::DIMENSION);

    // Create the collection.
    let collection = build_randomized_records(config::DIMENSION, config::DATABASE_SIZE);

    // Benchmark the search speed.
    let routine = || {
        black_box(collection.search(&vector, 10).unwrap());
    };

    criterion.bench_function(id, |bencher| bencher.iter(routine));
}

criterion_group!(bench, bench_search_collection);
criterion_main!(bench);
