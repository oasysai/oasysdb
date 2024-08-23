use super::*;
use proto::data_node_server::DataNode as ProtoDataNode;

type CoordinatorURL = Box<str>;

/// Data node server definition.
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

    /// Return the configured database URL.
    pub fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }

    /// Return the coordinator URL for this data node.
    pub fn coordinator_url(&self) -> &CoordinatorURL {
        &self.coordinator_url
    }
}

impl ProtoDataNode for Arc<DataNode> {}
