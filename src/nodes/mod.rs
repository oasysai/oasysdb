mod coordinator;
mod data;

// Re-export types from submodules.
pub use coordinator::*;
pub use data::*;

type DatabaseURL = Url;
type ServerResult<T> = StandardResult<Response<T>, Status>;

// Import common modules below.
use crate::postgres::*;
use crate::protos;
use crate::types::*;
use sqlx::{Connection, PgConnection};
use std::net::SocketAddr;
use std::result::Result as StandardResult;
use std::sync::Arc;
use tonic::{async_trait, Request, Response, Status};
use url::Url;
