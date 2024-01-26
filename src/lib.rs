//! ![Oasys](https://i.postimg.cc/GtwK53vF/banner.png)

mod db;
mod func;

/// The vector database storing collections.
pub use db::database;
/// The collection of vectors and its data.
pub use func::collection;
/// Types for the vectors.
pub use func::vector;

#[cfg(test)]
mod tests;
