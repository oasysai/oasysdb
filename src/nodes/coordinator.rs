use super::*;
use protos::coordinator_node_server::CoordinatorNode as ProtoCoordinatorNode;

/// Coordinator node definition.
#[derive(Debug)]
pub struct CoordinatorNode {
    database_url: DatabaseURL,
}

impl CoordinatorNode {
    /// Create a new coordinator node instance.
    pub async fn new(database_url: impl Into<DatabaseURL>) -> Self {
        let database_url = database_url.into();

        // TODO: connect and setup schema if needed.
        let connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        // TODO: create new or restore state from database.

        Self { database_url }
    }

    /// Return the configured database URL.
    pub fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }
}

#[async_trait]
impl ProtoCoordinatorNode for Arc<CoordinatorNode> {
    async fn register_node(
        &self,
        _request: Request<protos::RegisterNodeRequest>,
    ) -> ServerResult<()> {
        Ok(Response::new(()))
    }
}
