use jemalloc_ctl::stats::allocated;
use jemallocator::Jemalloc;
use oasysdb::collection::*;

#[global_allocator]
static ALLOC: Jemalloc = Jemalloc;

fn main() {
    // Matches the configuration in bench/main.rs.
    let len = 1_000_000;
    let dimension = 128;

    // Build the vector collection.
    let records = Record::many_random(dimension, len);
    let config = Config::default();
    Collection::build(&config, &records).unwrap();

    // Measure the memory usage.
    let memory = allocated::read().unwrap();
    let size_mb = memory as f32 / (1024.0 * 1024.0);

    println!("For {} vector records of dimension {}", len, dimension);
    println!("Memory usage: {:.0}MB", size_mb);
}
