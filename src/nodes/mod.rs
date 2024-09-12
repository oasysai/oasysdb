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
use sqlx::{Connection, PgConnection};
use std::net::SocketAddr;
use std::result::Result as StandardResult;
use std::sync::Arc;
use tonic::{async_trait, Request, Response, Status};
use url::Url;

/// Node server trait for common functionality.
///
/// This trait provides common functionality for both coordinator and data
/// node implementations allowing us to write less duplicate code.
#[async_trait]
trait NodeExt {
    /// Return the database URL connected to the node.
    fn database_url(&self) -> &DatabaseURL;

    /// Return connection to the node's Postgres database.
    async fn connect(&self) -> Result<PgConnection, Status> {
        PgConnection::connect(self.database_url().as_ref())
            .await
            .map_err(|_| Status::internal("Failed to connect to Postgres"))
    }
}
