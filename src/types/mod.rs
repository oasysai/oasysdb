// Initialize modules without publicizing them.
mod metric;
mod record;
mod vector;

// Re-export types from the modules.
pub use metric::*;
pub use record::*;
pub use vector::*;

// Import common dependencies below.
use crate::protos;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tonic::Status;
