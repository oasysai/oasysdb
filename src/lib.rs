//! OasysDB

mod db;
mod ix;

pub use db::database;
pub use ix::{index, vector};

#[cfg(test)]
mod tests;
