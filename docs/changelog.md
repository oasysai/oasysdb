# v0.4.0

### What's Changed

- **CONDITIONAL BREAKING CHANGE**: Add an option to configure distance for the vector collection via `Config` struct. The new field `distance` can be set using the `Distance` enum. This includes Euclidean, Cosine, and Dot distance metrics. The default distance metric is Euclidean. This change is backward compatible if you are creating a config using the `Config::default()` method. Otherwise, you need to update the config to include the distance metric.

  ```rs
  let config = Config {
      ...
      distance: Distance::Cosine,
  };
  ```

- With the new distance metric feature, now, you can set a `relevancy` threshold for the search results. This will filter out the results that are below or above the threshold depending on the distance metric used. This feature is disabled by default which is set to -1.0. To enable this feature, you can set the `relevancy` field in the `Collection` struct.

  ```rs
  ...
  let mut collection = Collection::new(&config)?;
  collection.relevancy = 3.0;
  ```

- Add a new method `Collection::insert_many` to insert multiple vector records into the collection at once. This method is more optimized than using the `Collection::insert` method in a loop.

### Contributors

- @noneback
- @edwinkys

### Full Changelog

https://github.com/oasysai/oasysdb/compare/v0.3.0...v0.4.0

# v0.3.0

This release introduces a BREAKING CHANGE to one of the method from the `Database` struct. The `Database::create_collection` method has been removed from the library due to redundancy. The `Database::save_collection` method can be used to create a new collection or update an existing one. This change is made to simplify the API and to make it more consistent with the other methods in the `Database` struct.

### What's Changed

- **BREAKING CHANGE**: Removed the `Database::create_collection` method from the library. To replace this, you can use the code snippet below:

  ```rs
  // Before: this creates a new empty collection.
  db.create_collection("vectors", None, Some(records))?;

  // After: create new or build a collection then save it.
  // let collection = Collection::new(&config)?;
  let collection = Collection::build(&config, &records)?;
  db.save_collection("vectors", &collection)?;
  ```

- Added the `Collection::list` method to list all the vector records in the collection.
- Created a full Python binding for OasysDB which is available on PyPI. This allows you to use OasysDB directly from Python. The Python binding is available at https://pypi.org/project/oasysdb.

### Contributors

- @edwinkys
- @Zelaren
- @FebianFebian1

### Full Changelog

https://github.com/oasysai/oasysdb/compare/v0.2.1...v0.3.0

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
