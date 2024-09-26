#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../readme.md")]

mod protos;
mod types;

pub use protos::database_client::DatabaseClient as Database;
