mod coordinator;
mod data;

// Re-export types from submodules.
pub use coordinator::*;
pub use data::*;

type DatabaseURL = Box<str>;

// Import common modules below.
use crate::protos as proto;
use std::sync::Arc;
