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
use futures::stream::StreamExt;
use sqlx::FromRow;
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
    /// Return the parameters configured for the node.
    fn params(&self) -> &NodeParameters;

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

    async fn find_nearest_cluster(
        &self,
        vector: &Vector,
    ) -> Result<Option<Cluster>, Status> {
        let mut conn = self.connect().await?;
        let cluster_table = self.schema().cluster_table();
        let query = format!("SELECT id, centroid FROM {cluster_table}");
        let mut stream = sqlx::query(&query).fetch(&mut conn);

        let mut nearest_distance = f32::MAX;
        let mut nearest_cluster: Option<Cluster> = None;
        while let Some(row) = stream.next().await {
            let row = row.map_err(|e| {
                let message = format!("Failed to fetch cluster: {}", e);
                Status::internal(message)
            })?;

            let cluster = Cluster::from_row(&row)
                .map_err(|e| Status::internal(e.to_string()))?;

            let centroid = &cluster.centroid;
            let dist = self.params().metric.distance(vector, centroid);
            if dist < nearest_distance {
                nearest_distance = dist;
                nearest_cluster = Some(cluster);
            }
        }

        Ok(nearest_cluster)
    }

    async fn insert_cluster(&self, centroid: &Vector) -> Result<(), Status> {
        let mut conn = self.connect().await?;
        let cluster_table = self.schema().cluster_table();
        sqlx::query(&format!(
            "INSERT INTO {cluster_table} (centroid)
            VALUES ($1)"
        ))
        .bind(centroid.as_slice())
        .execute(&mut conn)
        .await
        .map_err(|_| Status::internal("Failed to insert cluster"))?;

        Ok(())
    }
}
