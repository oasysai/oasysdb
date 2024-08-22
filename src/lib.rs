#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![doc = include_str!("../readme.md")]

/// Module for machine learning algorithms.
pub mod ml;
/// Module for OasysDB native types and data structures.
pub mod types;

/// Module for gRPC services and clients.
#[allow(clippy::all)]
pub mod protos {
    tonic::include_proto!("coordinator");
    tonic::include_proto!("data");
}
