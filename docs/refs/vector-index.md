# Vector Index

OasysDB provides a set of vector indexing algorithms that allow us to index
vectors for nearest neighbor search. Each index implementation has its own
characteristics and is suitable for different use cases.

Some common traits that all index implementations share are:

- **Incremental Operation**: The ability to modify records in the index
  individually without rebuilding the entire index. This includes common
  operations such as inserting, updating, and deleting records.
- **Isolated Data Storage**: Each index stored its own data separately from one
  another and the source table. This creates an isolated environment for the
  index to perform retrieval operations separately from the source table.
- **On-disk Persistence**: This trait allows us to persist the index to a file
  and restore it later. This is especially useful when we have an index built
  from a large dataset and we want to reuse it later without having to rebuild
  the index from scratch.

??? info "How Persistence Works"

    When we persist an index, we serialize the index data to a
    little-endian byte format via the `bincode` crate and write it to a file.
    When we restore the index later on, we read the byte data from the file
    and deserialize it back as an index object.

## Index Implementations

OasysDB provides the following index implementations:

### Flat Index

The Flat Index is a simple index that stores vectors in a flat list. When we
search for nearest neighbors in the index, the Flat Index will scan through all
vectors in the index and return the nearest neighbors based on the query vector.

This index is also known as the brute-force index.

??? note "Search Complexity: O(DN)"

    - **D**: Dimensionality of the vectors.
    - **N**: Number of vectors in the index.

### IVFPQ Index

The IVFPQ (Inverted File with Product Quantization) Index is a more advanced
index that uses a combination of inverted files and product quantization to
speed up the nearest neighbor search while maintaining an exceptional memory
efficiency.

Depending on the configuration, we can customize the index to meet the
performance requirement for our use case. We can adjust the number of clusters,
sub-quantizers, and other parameters to balance recall, memory usage, and search
speed.

??? note "Search Complexity: O(DK)"

    - **D**: Dimensionality of the vectors.
    - **K**: Number of vectors in the cluster to explore.

    This calculation will vary depending on the number of clusters in the index
    and clusters to explore during the search. This complexity also depends on
    the time it takes to decode the quantized vectors.

    Note: This is a rough estimation of the search complexity.

## Fine-grained Operations

When we run OasysDB as an embedded database directly in our application, we gain
access to the low-level index implementations. These implementations allow us to
have a more fine-grained control over the data in the index.

These are the most notable operations that we can perform with the index:

- Creating a new index with custom parameters.
- Building the index from a set of records.
- Inserting records into the index incrementally.
- Updating records in the index.
- Deleting records from the index.
- Searching for nearest neighbors in the index.
- Persisting the index to a file.
- Restoring the index from a file.

These operations allow us to use an index implementation on its own, without
having to rely on the Database interface. For more detailed documentation on the
index implementations, please refer to OasysDB's Rust API documentation.

<!-- prettier-ignore -->
[:fontawesome-brands-rust: Docs.rs](https://docs.rs/oasysdb/latest/oasysdb/){ .md-button .md-button--primary .odb-button }
