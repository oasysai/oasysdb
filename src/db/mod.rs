use crate::types::error::Error;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;
use tonic::{Request, Response, Status};

pub mod collection;
pub mod database;
