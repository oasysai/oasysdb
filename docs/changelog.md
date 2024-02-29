# v0.2.1

### What's Changed

- `Metadata` enum can now be accessed publicly using `oasysdb::metadata::Metadata`. This allows users to use `match` statements to extract the data from it.
- Added a `prelude` module that re-exports the most commonly used types and traits. This makes it easier to use the library by importing the prelude module by `use oasysdb::prelude::*`.

### Contributors

- @edwinkys

### Full Changelog

https://github.com/oasysai/oasysdb/compare/v0.2.0...v0.2.1

# v0.2.0

### What's Changed

- For `Collection` struct, the generic parameter `D` has been replaced with `Metadata` enum which allows one collection to store different types of data as needed.
- The `Vector` now uses `Vec<f32>` instead of `[f32, N]` which removes the `N` generic parameter from the `Vector` struct. Since there is a chance of using different vector dimensions in the same collection with this change, An additional functionality is added to the `Collection` to make sure that the vector dimension is uniform.
- The `M` generic parameter in the `Collection` struct has been replaced with a constant of 32. This removes the flexibility to tweak the indexing configuration for this value. But for most use cases, this value should be sufficient.
- Added multiple utility functions to structs such as `Record`, `Vector`, and `Collection` to make it easier to work with the data.

### Contributors

- @edwinkys

### Full Changelog

https://github.com/oasysai/oasysdb/compare/v0.1.0...v0.2.0

# v0.1.0

### What's Changed

- OasysDB release as an embedded vector database available directly via `cargo add oasysdb` command.
- Using HNSW algorithm implementation for the collection indexing along with Euclidean distance metrics.
- Incremental updates on the vector collections allowing inserts, deletes, and modifications without rebuilding the index.
- Add a benchmark on the collection search functionality using SIFT dataset that can be run using `cargo bench` command.

### Contributors

- @edwinkys

### Full Changelog

https://github.com/oasysai/oasysdb/commits/v0.1.0
