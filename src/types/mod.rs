// Initialize modules without publicizing them.
mod vector;

// Re-export types from the modules.
pub use vector::*;

// Import common dependencies below.
use serde::{Deserialize, Serialize};
