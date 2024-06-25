use crate::types::error::{Error, ErrorCode};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, RwLock as Lock};
use tonic::{Request, Response, Status};

pub mod collection;
pub mod database;
pub mod database_service;

use collection::*;
