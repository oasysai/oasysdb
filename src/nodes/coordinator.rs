use super::*;
use crate::protos::coordinator_node_server as proto;

/// Coordinator node definition.
#[allow(dead_code)]
#[derive(Debug)]
pub struct CoordinatorNode {
    database_url: DatabaseURL,
}

impl CoordinatorNode {
    /// Create a new coordinator node instance.
    pub fn new(database_url: impl Into<DatabaseURL>) -> Self {
        Self { database_url: database_url.into() }
    }
}

impl proto::CoordinatorNode for Arc<CoordinatorNode> {}
