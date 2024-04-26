![OasysDB Use Case](https://i.postimg.cc/k4x4Q55k/banner.png)

[![GitHub Stars](https://img.shields.io/github/stars/oasysai/oasysdb?label=Stars&logo=github&style=for-the-badge&color=%23fcd34d)](https://github.com/oasysai/oasysdb)
[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&style=for-the-badge)](https://discord.gg/bDhQrkqNP4)
[![Crates.io](https://img.shields.io/crates/d/oasysdb?style=for-the-badge&logo=rust&label=Crates.io&color=%23f43f5e)](https://crates.io/crates/oasysdb)
[![PyPI](https://img.shields.io/pypi/dm/oasysdb?style=for-the-badge&label=PyPI&logo=python&logoColor=ffffff&color=%232dd4bf)](https://pypi.org/project/oasysdb/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](https://opensource.org/licenses/Apache-2.0)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](/docs/code_of_conduct.md)

# 👋 Meet OasysDB

OasysDB is a **lightweight** and **easy-to-use** embedded vector database written in Rust. With its simple API, it requires no learning curve to understand and use. OasysDB also requires no server setup and configuration. It is designed to be embedded directly inside your AI application simply by adding it as a dependency.

```bash
# Rust via Crates.io
cargo add oasysdb

# Python via PyPI
pip install oasysdb
```

## Use Cases

OasysDB is very flexible! You can use it for systems related with vector similarity search such as:

- Local RAG (Retrieval-Augmented Generation) pipeline with an LLM and embedding model to generate a context-aware output.
- Image similarity search engine to find similar images based on their semantic content. [See Python demo](https://colab.research.google.com/drive/15_1hH7jGKzMeQ6IfnScjsc-iJRL5XyL7?usp=sharing).
- Real-time product recommendation system to suggest similar products based on the product features or user preferences.
- Add your use case here 😁

## Features

### Core Features

🔸 **Embedded Database**: Zero setup & no server required.\
🔸 **Optional Persistence**: In-memory or disk-based collection.\
🔸 **Incremental Ops**: Modify vectors without rebuilding indexes.\
🔸 **Flexible Schema**: Store additional metadata for each vector.

### Technical Features

🔹 **Fast HNSW**: Efficient approximate vector similarity search.\
🔹 **Configurable Metric**: Use Euclidean, Cosine, or other metric.\
🔹 **Parallel Processing**: Multi-threaded & SIMD optimized calculation.\
🔹 **Built-in Incremental ID**: No headache vector record management.

## Design Philosophy

OasysDB is designed to be boring 😂

Simple and easy to use API with no learning curve. No worries about setting up a server or configuring the database. We want you to forget about the vector database stuff and actually focus on building your AI application fast.

Read more about the design philosophy of OasysDB in the [Comprehensive Guide](/docs/guide.md).

# 🚀 Quickstart with Rust

![Rust-Banner.png](https://i.postimg.cc/NMCwFBPd/Rust-Banner.png)

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

    for res in result {
        let (id, distance) = (res.id, res.distance);
        println!("{distance:.5} | ID: {id}");
    }
}
```

## Dealing with Metadata Types

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

# 🚀 Quickstart with Python

![Python-Banner.png](https://i.postimg.cc/rp1qjBZJ/Python-Banner.png)

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

If you want to learn more about using OasysDB for real-world applications, you can check out the this Google Colab notebook which demonstrates how to use OasysDB to build a simple image similarity search engine: [Image Search Engine with OasysDB](https://colab.research.google.com/drive/15_1hH7jGKzMeQ6IfnScjsc-iJRL5XyL7?usp=sharing)

# 🎯 Benchmarks

OasysDB uses a built-in benchmarking suite using Rust's [Criterion](https://docs.rs/criterion) crate which we use to measure the performance of the vector database.

Currently, the benchmarks are focused on the performance of the collection's vector search functionality. We are working on adding more benchmarks to measure the performance of other operations.

If you are curious and want to run the benchmarks, you can use the command below to run the benchmarks. If you do run it, please share the results with us 😉

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

| Collection size | Embedding dimension | Memory usage | Search time |
| :-------------- | :-----------------: | -----------: | ----------: |
| 10,000          |         128         |          7MB |    248.73µs |
| 1,000,000       |         128         |        569MB |    555.46µs |
| 10,000          |         768         |        302MB |    705.83µs |
| 1,000,000       |         768         |      3,011MB |      1.36ms |
| 1,000,000       |        3,072        |          N/A |      3.07ms |
| 1,000,000       |        4,096        |          N/A |      3.87ms |

These results are from a machine with an Apple M3 CPU with 128GB of RAM. The dataset used for the benchmarks is a random dataset generated by the `Record::many_random` function with additional random `usize` as its metadata and a random search vector.

# 🤝 Contributing

The easiest way to contribute to this project is to star this project and share it with your friends. This will help us grow the community and make the project more visible to others.

If you want to go further and contribute your expertise, we will gladly welcome your code contributions. For more information and guidance about this, please see [contributing.md](/docs/contributing.md).

If you have deep experience in the space but don't have the free time to contribute codes, we also welcome advices, suggestions, or feature requests. We are also looking for advisors to help guide the project direction and roadmap.

If you are interested about the project in any way, please join us on [Discord](https://discord.gg/bDhQrkqNP4). Help us grow the community and make OasysDB better 😁

## Code of Conduct

We are committed to creating a welcoming community. Any participant in our project is expected to act respectfully and to follow the [Code of Conduct](/docs/code_of_conduct.md).

## Disclaimer

This project is still in the early stages of development. We are actively working on it and we expect the API and functionality to change. We do not recommend using this in production yet.
