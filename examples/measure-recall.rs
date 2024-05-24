// Note: This example measures the recall rate of the HNSW index.
// This might not reflect the actual performance of the index, as the
// recall rate is highly dependent on the quality of the data and the
// query distribution.

use oasysdb::prelude::*;

// High-level collection configuration.
const DIMENSION: usize = 1536;
const COLLECTION_SIZE: usize = 1000;

// HNSW configuration.
const EF_CONSTRUCTION: usize = 128;
const EF_SEARCH: usize = 64;
const ML: f32 = 0.2885;
const DISTANCE: &str = "euclidean";

// Query configuration.
const N_QUERIES: usize = 100;
const K: usize = 3;

fn main() {
    // Build a collection.
    let records = Record::many_random(DIMENSION, COLLECTION_SIZE);
    let config = Config::new(EF_CONSTRUCTION, EF_SEARCH, ML, DISTANCE).unwrap();
    let collection = Collection::build(&config, &records).unwrap();

    // Query the collection.
    let mut results = Vec::new();
    let mut true_results = Vec::new();

    for _ in 0..N_QUERIES {
        let query = Vector::random(DIMENSION);
        let result = collection.search(&query, K).unwrap();
        let true_result = collection.true_search(&query, K).unwrap();

        results.push(result);
        true_results.push(true_result);
    }

    // Measure recall.
    let mut correct = 0;
    for _ in 0..N_QUERIES {
        let result = results.pop().unwrap();
        let true_result = true_results.pop().unwrap();

        for r in result.iter() {
            if true_result.contains(r) {
                correct += 1;
            }
        }
    }

    let recall = (100 * correct) as f64 / (N_QUERIES * K) as f64;
    println!("Recall Rate: {recall:.2}%");
}
