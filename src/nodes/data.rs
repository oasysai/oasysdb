use super::*;
use protos::data_node_server::DataNode as ProtoDataNode;

type NodeName = Box<str>;
type CoordinatorURL = Box<str>;

/// Data node server definition.
#[derive(Debug)]
pub struct DataNode {
    name: NodeName,
    database_url: DatabaseURL,
    coordinator_url: CoordinatorURL,
}

impl DataNode {
    /// Create a new data node instance.
    pub async fn new(
        name: impl Into<NodeName>,
        database_url: impl Into<DatabaseURL>,
        coordinator_url: impl Into<CoordinatorURL>,
    ) -> Self {
        let name = name.into();
        let database_url = database_url.into();
        let coordinator_url = coordinator_url.into();

        // TODO: connect and setup schema if needed.

        // Register with the coordinator.

        Self { name, database_url, coordinator_url }
    }

    /// Return the name configured for this data node.
    pub fn name(&self) -> &NodeName {
        &self.name
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

#[async_trait]
impl ProtoDataNode for Arc<DataNode> {}
