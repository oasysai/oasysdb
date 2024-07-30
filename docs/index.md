# Welcome to OasysDB 🎉

First of all, thank you for considering to use OasysDB! We hope that OasysDB
will help you build your AI projects faster and more efficiently.

Before you dive deep into OasysDB, these are a few things you should know about
OasysDB and why you should or shouldn't use it.

## What is OasysDB?

OasysDB is a hybrid vector database that allows you to have a vector index layer
for similarity search with SQL database as your primary storage. For real-time
and constantly changing data, this means you can use SQL databases like
PostgreSQL, MySQL, or SQLite which offer ACID compliance and strong
transactional support as your primary storage layer and only use OasysDB for
similarity search.

![OasysDB Use Case](https://odb-assets.s3.amazonaws.com/banners/0.7.0.png)

## Features

<div class="grid cards" markdown>

<!-- prettier-ignore -->
- **SQL Storage Layer**

    OasysDB allows you to consolidate your vector data with other operational
    data in a single SQL database without impacting the performance of your
    SQL database.

- **Flexible Indexing**

    You can pick your own poison by choosing indexing algorithms that fit your
    use case like Flat (Brute Force) or IVFPQ. You can also configure the index
    to fit your performance requirements.

- **Multi-index Support**

    Depending on your use case and setup, you can create multiple vector
    indices for different vector columns from the same table to improve your
    search performance.

- **Pre-filtering**

    In addition to post-filtering, OasysDB supports pre-filtering allowing you
    to create an index for a subset of your data to narrow down the search
    space before performing the ANN search.

</div>

## Why not OasysDB?

- **Fully In-memory**: OasysDB stores the entire index in memory which means
  that the size of your index is limited by the memory available on your
  machine. If you have a large dataset over 10M vectors, you may want to
  consider using a disk-based indexing algorithm.
- **Hybrid Solution**: OasysDB is a hybrid of SQL database and vector indexing
  layer. This means that you need to use a SQL database as your primary storage
  layer for OasysDB to be optimal. OasysDB, or any other vector databases for
  that matter, won't be able to replace a transactional database.
