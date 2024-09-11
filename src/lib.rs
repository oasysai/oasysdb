#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../readme.md")]

/// Module for coordinator and data node server implementations.
pub mod nodes;
/// Module for PostgreSQL database utilities.
pub mod postgres;
/// Module for OasysDB native types and data structures.
pub mod types;

/// Module for gRPC services and clients.
#[allow(missing_docs)]
#[allow(clippy::all)]
pub mod protos {
    tonic::include_proto!("coordinator");
    tonic::include_proto!("data");
}
