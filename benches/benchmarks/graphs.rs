use std::collections::HashMap;
use criterion::{criterion_group, Criterion};
use oasysdb::db::database::*;
use crate::benchmarks::utils;


fn create_database(path: String, dimension: usize) -> Database {
    // remove the database if it already exists
    let _ = std::fs::remove_dir_all(&path);
    let config = Config { path, dimension };
    let db = Database::new(config);
    db
}

fn add_values(db: &Database, query: &Vec<Embedding>) {
    // Add the values to the database
    for i in 0..query.len() {
        let embedding: Embedding = query[i].clone();
        let value = Value { embedding, data: HashMap::new() };
        db.set_value(&i.to_string(), value).unwrap();
    }
}

fn build_graph(db: &Database, name: &str, ef_construction: usize, ef_search: usize) {
    // Build the graph
    let config = GraphConfig { name: name.to_string(), ef_construction, ef_search, filter: None };
    db.create_graph(config).unwrap();
}

fn search_graph(db: &Database, name: &str, query: &Embedding, k: usize) {
    // Search the graph
    let _ = db.query_graph(name, query.clone(), k).unwrap();
}


fn bench_add_values(c: &mut Criterion) {
    // Load the query dataset
    let base = utils::load_base_dataset();
    let dimension = base[0].len();

    c.bench_function("add values (10_000 vector of dim 128)",
                     |b| {
                         b.iter_batched(
                             // Setup the database (remove the database if it already exists and create a new one)
                             || {
                                 let path = "data/benchmarks".to_string();
                                 create_database(path, dimension)
                             },
                             |db| {
                                 add_values(&db, &base);
                             },
                             criterion::BatchSize::SmallInput,
                         )
                     });

    // remove the database
    std::fs::remove_dir_all("data/benchmarks").unwrap();
}

fn bench_build_graph(c: &mut Criterion) {
    // Load the query dataset
    let base = utils::load_base_dataset();
    let dimension = base[0].len();

    let path = "data/benchmarks".to_string();
    let db = create_database(path, dimension);
    add_values(&db, &base[0..1_000].to_vec());


    c.bench_function("build graph (1_000 vector of dim 128, ef_construction=25, ef_search=25)",
                     |b| {
                         b.iter_batched(
                             // Setup the database (remove the database if it already exists and create a new one)
                             || {
                                 &db.delete_graph("test");
                             },
                             |_| {
                                 build_graph(&db, "test", 25, 25);
                             },
                             criterion::BatchSize::SmallInput,
                         )
                     });

    // remove the database
    std::fs::remove_dir_all("data/benchmarks").unwrap();
}

fn bench_search_graph(c: &mut Criterion) {
    // Load the query dataset
    let base = utils::load_base_dataset();
    let query = utils::load_query_dataset();
    let dimension = base[0].len();


    let path = "data/benchmarks".to_string();
    let db = create_database(path, dimension);
    add_values(&db, &base);
    build_graph(&db, "test", 25, 25);

    c.bench_function("search in graph (10_000 vector of dim 128, ef_construction=25, ef_search=25)",
                     |b| {
                         b.iter(
                             || {
                                 search_graph(&db, "test", &query[0], 10);
                             },
                         )
                     });

    // remove the database
    std::fs::remove_dir_all("data/benchmarks").unwrap();
}




criterion_group!(benches, bench_add_values, bench_build_graph, bench_search_graph);