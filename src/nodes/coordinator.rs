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
        let mut connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        create_schema(&mut connection, COORDINATOR_SCHEMA).await;
        create_cluster_table(&mut connection, COORDINATOR_SCHEMA).await;
        create_coordinator_connection_table(&mut connection).await;
        create_coordinator_subcluster_table(&mut connection).await;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::tests::*;

    #[tokio::test]
    async fn test_coordinator_node_new() {
        let db = database_url();
        let mut connection = PgConnection::connect(&db.to_string())
            .await
            .expect("Failed to connect to Postgres database");

        drop_schema(&mut connection, COORDINATOR_SCHEMA).await;
        CoordinatorNode::new(db).await;

        let schema = get_schema(&mut connection, COORDINATOR_SCHEMA).await;
        assert_eq!(schema, COORDINATOR_SCHEMA);

        let tables = get_tables(&mut connection, COORDINATOR_SCHEMA).await;
        assert_eq!(tables.len(), 3);
    }
}
