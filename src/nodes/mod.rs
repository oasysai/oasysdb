mod coordinator;
mod data;

// Re-export types from submodules.
pub use coordinator::*;
pub use data::*;

type DatabaseURL = Url;
type ServerResult<T> = StandardResult<Response<T>, Status>;

// Import common modules below.
use crate::postgres::*;
use crate::types::Vector;
use sqlx::FromRow;
use sqlx::{Connection, PgConnection};
use std::net::SocketAddr;
use std::result::Result as StandardResult;
use std::sync::Arc;
use tonic::{async_trait, Request, Response, Status};
use url::Url;
use uuid::Uuid;

/// Node server trait for common functionality.
///
/// This trait provides common functionality for both coordinator and data
/// node implementations allowing us to write less duplicate code.
#[async_trait]
trait NodeExt {
    /// Return the database URL connected to the node.
    fn database_url(&self) -> &DatabaseURL;

    /// Return the schema configuration of the node.
    fn schema(&self) -> &impl NodeSchema;

    /// Return connection to the node's Postgres database.
    async fn connect(&self) -> Result<PgConnection, Status> {
        PgConnection::connect(self.database_url().as_ref())
            .await
            .map_err(|_| Status::internal("Failed to connect to Postgres"))
    }

    /// Insert a new cluster into the node's database.
    async fn _insert_cluster(
        &self,
        conn: &mut PgConnection,
        centroid: &Vector,
    ) -> Result<Uuid, Status> {
        let id = Uuid::new_v4();
        let byte = bincode::serialize(centroid).map_err(|_| {
            Status::internal("Failed to serialize centroid vector")
        })?;

        let cluster_table = self.schema().cluster_table();
        sqlx::query(&format!(
            "INSERT INTO {cluster_table} (id, centroid)
            VALUES ($1, $2)"
        ))
        .bind(id)
        .bind(&byte)
        .execute(conn)
        .await
        .map_err(|_| Status::internal("Failed to insert a new cluster"))?;

        Ok(id)
    }
}
