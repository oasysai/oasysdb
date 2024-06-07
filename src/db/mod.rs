/// The vector database storing collections.
pub mod database;

use crate::collection::*;
use crate::func::err::{Error, ErrorKind};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "py")]
use pyo3::prelude::*;
