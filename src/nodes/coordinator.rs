use super::*;
use proto::coordinator_node_server::CoordinatorNode as ProtoCoordinatorNode;

/// Coordinator node definition.
#[derive(Debug)]
pub struct CoordinatorNode {
    database_url: DatabaseURL,
}

impl CoordinatorNode {
    /// Create a new coordinator node instance.
    pub fn new(database_url: impl Into<DatabaseURL>) -> Self {
        Self { database_url: database_url.into() }
    }

    /// Return the configured database URL.
    pub fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }
}

impl ProtoCoordinatorNode for Arc<CoordinatorNode> {}
