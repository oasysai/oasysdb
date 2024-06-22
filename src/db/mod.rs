use crate::types::error::Error;
use rayon::prelude::*;
use tonic::{Request, Response, Status};

pub mod collection;
pub mod database;
