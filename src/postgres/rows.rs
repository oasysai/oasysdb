#![allow(missing_docs)]

use super::*;
use crate::protoc;
use crate::types::Vector;
use sqlx::postgres::PgRow;
use sqlx::Error as DatabaseError;
use sqlx::Result as DatabaseResult;
use sqlx::{FromRow, Row};
use std::net::SocketAddr;
use uuid::Uuid;

type NodeName = Box<str>;

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

impl From<protoc::NodeParameters> for NodeParameters {
    fn from(value: protoc::NodeParameters) -> Self {
        Self {
            metric: value.metric().into(),
            dimension: value.dimension as usize,
            density: value.density as usize,
        }
    }
}

impl From<NodeParameters> for protoc::NodeParameters {
    fn from(value: NodeParameters) -> Self {
        let metric = match value.metric {
            Metric::Cosine => protoc::Metric::Cosine,
            Metric::Euclidean => protoc::Metric::Euclidean,
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
/// - count: Number of sub-clusters in the node.
#[derive(Debug)]
pub struct NodeConnection {
    pub name: NodeName,
    pub address: SocketAddr,
    pub count: usize,
}

impl FromRow<'_, PgRow> for NodeConnection {
    fn from_row(row: &PgRow) -> DatabaseResult<Self> {
        let name = row.try_get("name")?;
        let count = row.try_get::<i32, _>("count")? as usize;

        let address = row
            .try_get::<String, _>("address")?
            .parse::<SocketAddr>()
            .map_err(|_| DatabaseError::Decode("node address".into()))?;

        Ok(Self { name, address, count })
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
    pub count: usize,
}

impl FromRow<'_, PgRow> for Cluster {
    fn from_row(row: &PgRow) -> DatabaseResult<Self> {
        let id = row.try_get("id")?;
        let count = row.try_get::<i32, _>("count")? as usize;

        let centroid = {
            let bytea = row.try_get::<Vec<u8>, _>("centroid")?;
            bincode::deserialize(&bytea)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
        };

        Ok(Self { id, centroid, count })
    }
}
