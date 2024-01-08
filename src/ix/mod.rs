mod hyperplane;
mod tree;

pub mod index;
pub mod vector;

use hyperplane::*;
use tree::*;
use vector::*;

// External deps.
use dashmap::DashSet;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use std::cmp::min;
use std::collections::HashMap;
