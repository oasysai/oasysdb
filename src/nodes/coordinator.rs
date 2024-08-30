use super::*;
use protos::coordinator_node_server::CoordinatorNode as ProtoCoordinatorNode;

/// Coordinator node definition.
///
/// Coordinator nodes are responsible for managing the functionality of OasysDB
/// clusters. Data nodes can register with a coordinator node to join a cluster
/// which will horizontally extend OasysDB processing capabilities.
#[derive(Debug)]
pub struct CoordinatorNode {
    database_url: DatabaseURL,
    schema: CoordinatorSchema,
}

impl CoordinatorNode {
    /// Create a new coordinator node instance.
    pub async fn new(database_url: impl Into<DatabaseURL>) -> Self {
        let database_url = database_url.into();
        let mut connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        let schema = CoordinatorSchema::new();
        schema.create_schema(&mut connection).await;
        schema.create_all_tables(&mut connection).await;

        // TODO: create new or restore state from database.

        Self { database_url, schema }
    }

    /// Return the configured database URL.
    pub fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }

    /// Return the schema configured for the coordinator node.
    pub fn schema(&self) -> &CoordinatorSchema {
        &self.schema
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
    use crate::postgres::test_utils::*;

    const COORDINATOR_SCHEMA: &str = "coordinator";

    #[tokio::test]
    async fn test_coordinator_node_new() {
        let db = database_url();
        let mut connection = PgConnection::connect(&db.to_string())
            .await
            .expect("Failed to connect to Postgres database");

        drop_schema(&mut connection, COORDINATOR_SCHEMA).await;
        CoordinatorNode::new(db).await;

        let tables = get_tables(&mut connection, COORDINATOR_SCHEMA).await;
        assert_eq!(tables.len(), 4);
    }
}
