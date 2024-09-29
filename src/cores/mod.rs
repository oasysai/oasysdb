// Initialize the modules without making them public.
mod database;
mod index;
mod storage;

// Re-export types from the modules.
pub use database::*;
pub use index::*;
pub use storage::*;

// Import common dependencies below.
use crate::types::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};
