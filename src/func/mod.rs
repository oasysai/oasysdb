/// The collection of vectors and their data.
pub mod collection;
/// Enum for the collection distance functions.
pub mod distance;
/// Error types for the database.
pub mod err;
/// Types for the metadata.
pub mod metadata;
/// Types for the vectors.
pub mod vector;

// Internal modules.
mod utils;

use collection::*;
use distance::*;
use err::*;
use metadata::*;
use utils::*;
use vector::*;

// External dependencies.
use ordered_float::OrderedFloat;
use parking_lot::*;
use rand::random;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use simsimd::SpatialSimilarity;
use std::cmp::*;
use std::collections::{BinaryHeap, HashMap};
use std::ops::{Deref, Index};

#[cfg(feature = "py")]
use pyo3::prelude::*;

// This code is inspired by the HNSW implementation in the
// Instant Distance library and modified to fit the needs
// of this project.
// https://github.com/instant-labs/instant-distance
