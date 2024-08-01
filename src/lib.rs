#![warn(missing_docs)]
#![warn(unused_qualifications)]
#![doc = include_str!("../readme.md")]
#![doc(html_favicon_url = "https://i.postimg.cc/W3T230zk/favicon.png")]
#![doc(html_logo_url = "https://i.postimg.cc/Vv0HPVwB/logo.png")]

pub(crate) mod utils;

/// Primary module for vector database operations.
pub mod db;
/// Module for managing database indices and related types.
pub mod indices;
/// Convenience re-exports of the public APIs.
pub mod prelude;
/// Database utility types and functions.
pub mod types;
