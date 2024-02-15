/// The vector database storing collections.
pub mod database;

use crate::collection::*;
use crate::func::err;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sled::Db;
use std::error::Error;
use std::fs::remove_dir_all;
use std::path::Path;
