/// The vector database storing collections.
pub mod database;

use crate::collection::*;
use crate::func::err::Error;
use sled::Db;
use std::fs::{create_dir_all, remove_dir_all, remove_file, OpenOptions};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

#[cfg(feature = "py")]
use pyo3::prelude::*;
