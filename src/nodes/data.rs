use super::*;
use crate::protos::data_node_server as proto;

type CoordinatorURL = Box<str>;

/// Data node server definition.
#[allow(dead_code)]
#[derive(Debug)]
pub struct DataNode {
    database_url: DatabaseURL,
    coordinator_url: CoordinatorURL,
}

impl DataNode {
    /// Create a new data node instance.
    pub fn new(
        database_url: impl Into<DatabaseURL>,
        coordinator_url: impl Into<CoordinatorURL>,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            coordinator_url: coordinator_url.into(),
        }
    }
}

impl proto::DataNode for Arc<DataNode> {}
