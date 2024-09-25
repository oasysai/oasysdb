#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../readme.md")]

/// Module for OasysDB native types and data structures.
pub mod types;

/// Module for database server's gRPC implementations.
#[allow(missing_docs)]
#[allow(clippy::all)]
pub mod protos {
    tonic::include_proto!("database");
}
