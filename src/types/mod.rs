// Initialize modules without publicizing them.
mod metric;
mod vector;

// Re-export types from the modules.
pub use metric::*;
pub use vector::*;

// Import common dependencies below.
use serde::{Deserialize, Serialize};
