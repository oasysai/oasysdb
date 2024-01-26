pub mod database;

use crate::collection::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sled::Db;
use std::fs::remove_dir_all;
use std::path::Path;
