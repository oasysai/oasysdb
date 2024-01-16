pub mod index;
pub mod vector;

// Internal modules.
mod utils;

use utils::*;
use vector::*;

// External dependencies.
use ordered_float::OrderedFloat;
use parking_lot::*;
use rand::*;
use rayon::iter::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::cmp::*;
use std::collections::{BinaryHeap, HashMap};
use std::ops::{Deref, Index};

// This code is inspired by the HNSW implementation in the
// Instant Distance library and modified to fit the needs
// of this project.
// https://github.com/instant-labs/instant-distance
