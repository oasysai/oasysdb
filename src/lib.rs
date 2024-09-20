#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../readme.md")]

/// Module for coordinator and data node server implementations.
pub mod nodes;
/// Module for PostgreSQL database utilities.
pub mod postgres;
/// Module for OasysDB native types and data structures.
pub mod types;

/// Module for coordinator node's gRPC types.
#[allow(missing_docs)]
#[allow(clippy::all)]
pub mod protoc {
    tonic::include_proto!("coordinator");
}

/// Module for data node's gRPC services and clients.
#[allow(missing_docs)]
#[allow(clippy::all)]
pub mod protod {
    tonic::include_proto!("data");
}
