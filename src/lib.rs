//! OasysDB

mod db;
mod ix;

pub use db::database;
pub use ix::index;

#[cfg(test)]
mod tests;
