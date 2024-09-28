// Initialize the modules without making them public.
mod database;
mod index;

// Re-export types from the modules.
pub use database::*;

// Import common dependencies below.
use crate::types::Metric;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};
