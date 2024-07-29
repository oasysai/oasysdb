![OasysDB Use Case](https://odb-assets.s3.amazonaws.com/banners/0.7.0.png)

[![GitHub Stars](https://img.shields.io/github/stars/oasysai/oasysdb?style=for-the-badge&logo=github&logoColor=%23000000&labelColor=%23fcd34d&color=%236b7280)](https://github.com/oasysai/oasysdb)
[![Discord](https://img.shields.io/badge/chat-%236b7280?style=for-the-badge&logo=discord&logoColor=%23ffffff&label=discord&labelColor=%237289da)][discord]
[![Documentation](https://img.shields.io/badge/read-6b7280?style=for-the-badge&label=oasysdb%20docs&labelColor=14b8a6)][docs]
[![Crates.io](https://img.shields.io/crates/d/oasysdb?style=for-the-badge&logo=rust&logoColor=%23000&label=crates.io&labelColor=%23fdba74&color=%236b7280)](https://crates.io/crates/oasysdb)

# Introducing OasysDB 👋

OasysDB is a hybrid vector database that allows you utilize relational databases
like SQLite and Postgres as a storage engine for your vector data without using
them to compute expensive vector operations.

This allows you to consolidate your data into a single database and ensure high
data integrity with the ACID properties of traditional databases while also
having a fast and isolated vector indexing layer.

For more details about OasysDB, please visit the
[Documentation](https://docs.oasysdb.com/).

# Quickstart 🚀

Currently, OasysDB is only available for Rust projects as an embedded database.
We are still working on implementing RPC APIs to allow you to use OasysDB in any
language as a standalone service.

OasysDB has 2 primary components: **Database** and **Index**.

- The Database is responsible for managing the vector indices and connecting the
  storage engine, the SQL database, to the indices as the data source. OasysDB
  uses SQLx to handle the SQL database operations.

- The Index implements a vector indexing algorithm and is responsible for
  storing and querying vectors. The functionality and algorithm of the index
  depends on the algorithm you choose when creating the index.

## Embedded in Rust

To use OasysDB as an embedded vector database in your Rust project, simply add
it to your Cargo.toml file or run the command below on your terminal:

```bash
cargo add oasysdb
```

When running OasysDB as an embedded database, you have access to both the
Database and Index interfaces. In a rare occassion that you're building a
project that doesn't utilize SQL, you can use the Index interface directly.
Otherwise, the quickstart guide below will show you how to use the Database
interface.

```rust no_run
// Use the prelude module to import all necessary functionalities.
use oasysdb::prelude::*;
use std::env;

// Open OasysDB database with connection to SQLite.
// Connection is required for new database but optional for existing ones.
let sqlite = "sqlite://sqlite.db";
let db = Database::open("odb_test", Some(sqlite)).unwrap();

// Create a new index with IVFPQ algorithm with default parameters.
let params = ParamsIVFPQ::default();
let algorithm = IndexAlgorithm::IVFPQ(params);
// Setup where the data of the index will come from.
let config = SourceConfig::new("table", "id", "vector");
db.create_index("index", algorithm, config).unwrap();

// Search the index for nearest neighbors of a query vector.
let query = vec![0.0; 128];
let filters = ""; // Optional SQL-like filter for the search.
let results = db.search_index("index", query, 10, filters).unwrap();
```

## More Resources

[![Discord](https://img.shields.io/badge/chat-%236b7280?style=for-the-badge&logo=discord&logoColor=%23ffffff&label=discord&labelColor=%237289da)][discord]
[![Documentation](https://img.shields.io/badge/read-6b7280?style=for-the-badge&label=oasysdb%20docs&labelColor=14b8a6)][docs]

There are more to OasysDB than what is shown in this Quickstart guide. Please
visit OasysDB's [Documentation][docs] for more information. In addition, if you
have any question or need help that needs immediate response, please join our
[Discord Server][discord] and I will try my best to help you as soon as
possible.

[docs]: https://docs.oasysdb.com
[discord]: https://discord.gg/bDhQrkqNP4
