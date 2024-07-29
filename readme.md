![OasysDB Use Case](https://odb-assets.s3.amazonaws.com/banners/0.7.0.png)

[![GitHub Stars](https://img.shields.io/github/stars/oasysai/oasysdb?style=for-the-badge&logo=github&logoColor=%23000000&labelColor=%23fcd34d&color=%236b7280)](https://github.com/oasysai/oasysdb)
[![Discord](https://img.shields.io/badge/chat-%236b7280?style=for-the-badge&logo=discord&logoColor=%23ffffff&label=discord&labelColor=%237289da)](https://discord.gg/bDhQrkqNP4)
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
