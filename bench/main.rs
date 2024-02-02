mod utils;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oasysdb::collection::{Collection, Config};
use oasysdb::vector::Vector;
use utils::{get_records, read_vectors};

fn build_collection(path: &str) -> Collection<usize, 128, 32> {
    let records = get_records(path).unwrap();
    let config = Config::default();
    Collection::build(&config, &records)
}

fn bench_search_collection(criterion: &mut Criterion) {
    let id = "Search collection";

    // Load the query data.
    let query_path = "data/siftsmall/siftsmall_query.fvecs";
    let query_data = read_vectors(query_path).unwrap();
    let query: [f32; 128] = query_data[0].as_slice().try_into().unwrap();

    // Create the collection.
    let base_path = "data/siftsmall/siftsmall_base.fvecs";
    let collection = build_collection(base_path);

    // Benchmark the search speed.
    let routine = || {
        black_box(collection.search(&Vector(query), 10));
    };

    criterion.bench_function(id, |bencher| bencher.iter(routine));
}

criterion_group!(bench, bench_search_collection);
criterion_main!(bench);
