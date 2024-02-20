mod utils;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oasysdb::collection::{Collection, Config};
use oasysdb::vector::Vector;
use utils::*;

fn build_collection(path: &str) -> Collection<usize> {
    let records = get_records(path).unwrap();
    let config = Config::default();
    Collection::build(&config, &records).unwrap()
}

fn bench_search_collection(criterion: &mut Criterion) {
    let id = "Search collection";

    // Download the dataset.
    download_siftsmall().unwrap();

    // Load the query data.
    let query_path = "data/siftsmall/siftsmall_query.fvecs";
    let query_data = read_vectors(query_path).unwrap();
    let query = &query_data[0];

    // Create the collection.
    let base_path = "data/siftsmall/siftsmall_base.fvecs";
    let collection = build_collection(base_path);

    // Benchmark the search speed.
    let routine = || {
        black_box(collection.search(&Vector(query.clone()), 10).unwrap());
    };

    criterion.bench_function(id, |bencher| bencher.iter(routine));
}

criterion_group!(bench, bench_search_collection);
criterion_main!(bench);
