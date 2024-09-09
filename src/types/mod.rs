// Initialize modules without publicizing them.
mod error;
mod metadata;
mod metric;
mod vector;

// Re-export types from the modules.
pub use error::*;
pub use metadata::*;
pub use metric::*;
pub use vector::*;

// Import common dependencies below.
use serde::{Deserialize, Serialize};
