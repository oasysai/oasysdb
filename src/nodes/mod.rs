mod coordinator;
mod data;

// Re-export types from submodules.
pub use coordinator::*;
pub use data::*;

type DatabaseURL = Url;
type CoordinatorURL = SocketAddr;
type ServerResult<T> = StandardResult<Response<T>, Status>;

// Import common modules below.
use crate::postgres::{CoordinatorSchema, DataSchema, NodeSchema};
use crate::protos;
use sqlx::{Connection, PgConnection};
use std::net::SocketAddr;
use std::result::Result as StandardResult;
use std::sync::Arc;
use tonic::{async_trait, Request, Response, Status};
use url::Url;
