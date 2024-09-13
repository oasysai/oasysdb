#![allow(missing_docs)]

use super::*;
use crate::protos;
use crate::types::Vector;
use sqlx::postgres::PgRow;
use sqlx::Error as DatabaseError;
use sqlx::Result as DatabaseResult;
use sqlx::{FromRow, Row};
use std::net::SocketAddr;
use uuid::Uuid;

type NodeName = Box<str>;

/// Coordinator node state object.
///
/// Fields:
/// - initialized: Whether the node index is initialized.
#[derive(Debug, Clone, FromRow)]
pub struct CoordinatorState {
    pub initialized: bool,
}

/// Node's index parameters.
///
/// Fields:
/// - metric: Formula used to calculate distance.
/// - dimension: Vector dimension.
/// - density: Number of records in each cluster.
#[derive(Debug, Clone, FromRow)]
pub struct NodeParameters {
    #[sqlx(try_from = "String")]
    pub metric: Metric,
    #[sqlx(try_from = "i32")]
    pub dimension: usize,
    #[sqlx(try_from = "i32")]
    pub density: usize,
}

impl NodeParameters {
    pub fn trace(&self) {
        tracing::info!("running the configured node parameters:");
        tracing::info!("metric: {}", self.metric.as_str());
        tracing::info!("dimension: {}", self.dimension);
        tracing::info!("density: {}", self.density);
    }
}

impl From<protos::RegisterNodeResponse> for NodeParameters {
    fn from(value: protos::RegisterNodeResponse) -> Self {
        Self {
            metric: value.metric().into(),
            dimension: value.dimension as usize,
            density: value.density as usize,
        }
    }
}

impl From<NodeParameters> for protos::RegisterNodeResponse {
    fn from(value: NodeParameters) -> Self {
        let metric = match value.metric {
            Metric::Cosine => protos::Metric::Cosine,
            Metric::Euclidean => protos::Metric::Euclidean,
        };

        Self {
            metric: metric as i32,
            dimension: value.dimension as i32,
            density: value.density as i32,
        }
    }
}

/// Details to connect to a data node.
///
/// Fields:
/// - name: Unique data node name.
/// - address: Node's address, IP address and port.
#[derive(Debug)]
pub struct NodeConnection {
    pub name: NodeName,
    pub address: SocketAddr,
}

impl FromRow<'_, PgRow> for NodeConnection {
    fn from_row(row: &PgRow) -> DatabaseResult<Self> {
        let name = row.try_get("name")?;
        let address = row
            .try_get::<String, _>("address")?
            .parse::<SocketAddr>()
            .map_err(|_| DatabaseError::Decode("node address".into()))?;

        Ok(Self { name, address })
    }
}

/// IVF index cluster reference.
///
/// Fields:
/// - id: Cluster's identifier, UUID.
/// - centroid: Centroid vector.
#[derive(Debug)]
pub struct Cluster {
    pub id: Uuid,
    pub centroid: Vector,
}

impl FromRow<'_, PgRow> for Cluster {
    fn from_row(row: &PgRow) -> DatabaseResult<Self> {
        let id = row.try_get("id")?;
        let centroid = {
            let bytea = row.try_get::<Vec<u8>, _>("centroid")?;
            bincode::deserialize(&bytea)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
        };

        Ok(Self { id, centroid })
    }
}
