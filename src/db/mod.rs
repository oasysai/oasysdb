/// The vector database storing collections.
pub mod database;

use crate::collection::*;
use crate::func::err::Error;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::hash_map::DefaultHasher;
use std::fs::{create_dir_all, remove_dir_all, remove_file, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "py")]
use pyo3::prelude::*;
