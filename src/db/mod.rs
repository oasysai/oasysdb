/// The vector database storing collections.
pub mod database;

use crate::collection::*;
use crate::func::err::Error;
use sled::Db;
use std::fs::remove_dir_all;
use std::path::Path;

#[cfg(feature = "py")]
use pyo3::prelude::*;
