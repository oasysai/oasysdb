![OasysDB Use Case](https://i.postimg.cc/k4x4Q55k/banner.png)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](https://opensource.org/licenses/Apache-2.0)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](/docs/code_of_conduct.md)
[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&style=for-the-badge)](https://discord.gg/bDhQrkqNP4)

## Introduction to OasysDB

OasysDB is an embeddable, efficient, and easy to use vector database. It is designed to be used as a library in your application. It is written in Rust and it uses Sled as its storage engine.

OasysDB implements HNSW as its indexing algorithm. It is a state-of-the-art algorithm that is used by many vector databases. It is fast, memory efficient, and it scales well to large datasets.

## Why OasysDB?

OasysDB is very flexible for use cases related with vector search. It offers 2 major features that make it stand out from other embeddable vector databases or libraries:

- Incremental operations on the collection index, which means that you can add, remove, or modify vectors from the index without having to rebuild it.
- Flexible persistence options. You can choose to persist the collection to disk or to keep it in memory.

## Quickstart

```rust
use oasysdb::database::Database;
use oasysdb::collection::*;
use oasysdb::vector::*;
use rand::random; // Utility

fn main() {
    // Utility functions to generate random vector records.
    let records = gen_records::<128>(100);

    // Open the database and create a collection.
    let mut db = Database::open("data/readme").unwrap();
    let collection: Collection<usize, 128, 32> =
        db.create_collection("vectors", None, Some(&records)).unwrap();

    // Utility function to generate a random vector.
    let query = gen_vector::<128>();
    let result = collection.search(&query, 5).unwrap();

    println!("Nearest neighbor ID: {}", result[0].id);
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
```

## Benchmarks

OasysDB has a built-in benchmarking suite using Rust's `criterion` crate that can be used to measure the performance of the vector database.

Currently, the benchmarks are focused on the performance of the collection's vector search functionality. We are working on adding more benchmarks to measure the performance of other operations.

If you are curious and want to run the benchmarks, you can use the following command which will download the benchmarking dataset and run the benchmarks:

```bash
cargo bench
```

## Disclaimer

This project is still in the early stages of development. We are actively working on it and we expect the API and functionality to change. We do not recommend using this in production yet.

We also don't have a benchmark yet. We are working on it and we will publish the results once we have them.

## Contributing

We welcome contributions from the community. Please see [contributing.md](/docs/contributing.md) for more information.

We are also looking for advisors to help guide the project direction and roadmap. If you are interested, please contact us via [Discord](https://discord.gg/bDhQrkqNP4) or alternatively, you can email me at edwin@oasysai.com.

## Code of Conduct

We are committed to creating a welcoming community. Any participant in our project is expected to act respectfully and to follow the [Code of Conduct](/docs/code_of_conduct.md).
