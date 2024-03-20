![OasysDB Use Case](https://i.postimg.cc/k4x4Q55k/banner.png)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](https://opensource.org/licenses/Apache-2.0)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](/docs/code_of_conduct.md)
[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&style=for-the-badge)](https://discord.gg/bDhQrkqNP4)
[![Crates.io](https://img.shields.io/crates/d/oasysdb?style=for-the-badge&label=Crates.io&color=%23f43f5e)](https://crates.io/crates/oasysdb)

# 👋 Meet OasysDB

OasysDB is a SQLite-inspired **lightweight** and **easy to use** embedded vector database. It is designed to be embedded directly inside your AI application. It is written in Rust and uses [Sled](https://docs.rs/sled) as its persistence storage engine to save vector collections to the disk.

OasysDB implements **HNSW** (Hierachical Navigable Small World) as its indexing algorithm. It is a state-of-the-art algorithm that is used by many vector databases. It is fast and scales well to large datasets.

## Why OasysDB?

OasysDB is very flexible for use cases related with vector search such as using RAG (Retrieval-Augmented Generation) method with an LLM to generate a context-aware output. These are some of the reasons why you might want to use OasysDB:

⭐️ **Embedded database**: OasysDB doesn't require you to set up a separate server and manage it. You can embed it directly into your application and use its simple API like a regular library.

⭐️ **Optional persistence**: You can choose to persist the vector collection to the disk or keep it in memory. By default, whenever you use a collection, it will be loaded to the memory to ensure that the search performance is high.

⭐️ **Incremental operations**: OasysDB allows you to add, remove, or modify vectors from collections without having to rebuild indexes. This allows for a more flexible and efficient approach on storing your vector data.

⭐ **Flexible schema**: Along with the vectors, you can store additional metadata for each vector. This is useful for storing information about the vectors such as the original text, image URL, or any other data that you want to associate with the vectors.

# ⚙️ Quickstart with Rust

To get started with OasysDB in Rust, you need to add `oasysdb` to your `Cargo.toml`. You can do so by running the command below which will add the latest version of OasysDB to your project.

```bash
cargo add oasysdb
```

After that, you can use the code snippet below as a reference to get started with OasysDB. In short, use `Collection` to store your vector records or search similar vector and use `Database` to persist a vector collection to the disk.

```rust
use oasysdb::prelude::*;

fn main() {
    // Vector dimension must be uniform.
    let dimension = 128;

    // Replace with your own data.
    let records = Record::many_random(dimension, 100);

    let mut config = Config::default();

    // Optionally set the distance function. Default to Euclidean.
    config.distance = Distance::Cosine;

    // Create a vector collection.
    let collection = Collection::build(&config, &records).unwrap();

    // Optionally save the collection to persist it.
    let mut db = Database::new("data/test").unwrap();
    db.save_collection("vectors", &collection).unwrap();

    // Search for the nearest neighbors.
    let query = Vector::random(dimension);
    let result = collection.search(&query, 5).unwrap();
    println!("Nearest ID: {}", result[0].id);
}
```

## Dealing with Metadata

In OasysDB, you can store additional metadata for each vector which is useful to associate the vectors with other data. The code snippet below shows how to insert the `Metadata` to the `Record` or extract it.

```rust
use oasysdb::prelude::*;

fn main() {
    // Inserting a metadata value into a record.
    let data: &str = "This is an example.";
    let vector = Vector::random(128);
    let record = Record::new(&vector, &data.into());

    // Extracting the metadata value.
    let metadata = record.data.clone();
    let data = match metadata {
        Metadata::Text(value) => value,
        _ => panic!("Data is not a text."),
    };

    println!("{}", data);
}
```

# 🐍 Quickstart with Python

OasysDB also provides a Python binding which allows you to add it directly to your project. You can install the Python library of OasysDB by running the command below:

```bash
pip install oasysdb
```

This command will install the latest version of OasysDB to your Python environment. After you're all set with the installation, you can use the code snippet below as a reference to get started with OasysDB in Python.

```python
from oasysdb.prelude import *


if __name__ == "__main__":
    # Open the database.
    db = Database("data/example")

    # Replace with your own records.
    records = Record.many_random(dimension=128, len=100)

    # Create a vector collection.
    config = Config.create_default()
    collection = Collection.from_records(config, records)

    # Optionally, persist the collection to the database.
    db.save_collection("my_collection", collection)

    # Replace with your own query.
    query = Vector.random(128)

    # Search for the nearest neighbors.
    result = collection.search(query, n=5)

    # Print the result.
    print("Nearest neighbors ID: {}".format(result[0].id))
```

# 🎯 Benchmarks

OasysDB uses a built-in benchmarking suite using Rust's [Criterion](https://docs.rs/criterion) crate which we use to measure the performance of the vector database.

Currently, the benchmarks are focused on the performance of the collection's vector search functionality. We are working on adding more benchmarks to measure the performance of other operations.

If you are curious and want to run the benchmarks, you can use the following command which will download the benchmarking dataset and run the benchmarks:

```bash
cargo bench
```

## Memory Usage

OasysDB uses HNSW which is known to be a memory hog compared to other indexing algorithms. We decided to use it because of its performance even when storing large datasets of vectors with high dimension.

In the future, we might consider adding more indexing algorithms to make OasysDB more flexible and to cater to different use cases. If you have any suggestions of which indexing algorithms we should add, please let us know.

Anyway, if you are curious about the memory usage of OasysDB, you can use the command below to run the memory usage measurement script. You can tweak the parameters in the `examples/measure-memory.rs` file to see how the memory usage changes.

```bash
cargo run --example measure-memory
```

## Quick Results

Even though the results may vary depending on the hardware and the dataset, we want to give you a quick idea of the performance of OasysDB. Here are some quick results from the benchmarks:

**10,000 vectors with 128 dimensions**

- Search time: 0.15 ms
- Memory usage: 6 MB

**1,000,000 vectors with 128 dimensions**

- Search time: 1.5 ms
- Memory usage: 600 MB

These results are from a machine with an Apple M2 CPU and 16 GB of RAM. The dataset used for the benchmarks is a random dataset generated by the `Record::many_random` function or SIFT datasets with additional random `usize` as its metadata.

# 🤝 Contributing

The easiest way to contribute to this project is to star this project and share it with your friends. This will help us grow the community and make the project more visible to others.

If you want to go further and contribute your expertise, we will gladly welcome your code contributions. For more information and guidance about this, please see [contributing.md](/docs/contributing.md).

If you have deep experience in the space but don't have the free time to contribute codes, we also welcome advices, suggestions, or feature requests. We are also looking for advisors to help guide the project direction and roadmap.

If you are interested about the project in any way, please join us on [Discord](https://discord.gg/bDhQrkqNP4). Help us grow the community and make OasysDB better 😁

## Code of Conduct

We are committed to creating a welcoming community. Any participant in our project is expected to act respectfully and to follow the [Code of Conduct](/docs/code_of_conduct.md).

## Disclaimer

This project is still in the early stages of development. We are actively working on it and we expect the API and functionality to change. We do not recommend using this in production yet.
