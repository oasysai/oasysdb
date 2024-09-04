#![allow(missing_docs)]

use super::*;
use crate::protos;
use sqlx::postgres::PgRow;
use sqlx::Error as DatabaseError;
use sqlx::Result as DatabaseResult;
use sqlx::{FromRow, Row};
use std::net::SocketAddr;

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

impl From<protos::NodeParameters> for NodeParameters {
    fn from(value: protos::NodeParameters) -> Self {
        Self {
            metric: value.metric().into(),
            dimension: value.dimension as usize,
            density: value.density as usize,
        }
    }
}

impl From<NodeParameters> for protos::NodeParameters {
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
