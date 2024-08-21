// Initialize modules without publicizing them.
mod error;
mod metric;
mod record;
mod vector;

// Re-export types from the modules.
pub use error::*;
pub use metric::*;
pub use record::*;
pub use vector::*;

// Import common dependencies below.
use serde::{Deserialize, Serialize};
