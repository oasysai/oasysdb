use super::*;
use protos::data_node_server::DataNode as ProtoDataNode;
use regex::Regex;

type NodeName = Box<str>;

/// Data node server definition.
#[derive(Debug)]
pub struct DataNode {
    name: NodeName,
    coordinator_url: CoordinatorURL,
    database_url: DatabaseURL,
}

impl DataNode {
    /// Create a new data node instance.
    pub async fn new(
        name: impl Into<NodeName>,
        coordinator_url: impl Into<CoordinatorURL>,
        database_url: impl Into<DatabaseURL>,
    ) -> Self {
        let name = name.into();
        let coordinator_url = coordinator_url.into();
        let database_url = database_url.into();

        // Validate node name: lowercase, alphanumeric, and underscores only.
        if !Regex::new("^[a-z0-9_]+$").unwrap().is_match(name.as_ref()) {
            let action = "Use lowercase letters, numbers, and underscores.";
            panic!("Invalid node name: {action}");
        }

        let mut connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        let schema = format!("{DATA_SCHEMA}{name}");
        create_schema(&mut connection, &schema).await;
        create_cluster_table(&mut connection, &schema).await;
        create_data_record_table(&mut connection, &schema).await;

        // Register with the coordinator.

        Self { name, coordinator_url, database_url }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::tests::*;

    #[tokio::test]
    async fn test_data_node_new() {
        let node_name = "49babacf";
        let schema_name = format!("{DATA_SCHEMA}{node_name}");

        let db = database_url();
        let coordinator = coordinator_url();

        let mut connection = PgConnection::connect(&db.to_string())
            .await
            .expect("Failed to connect to Postgres database");

        drop_schema(&mut connection, &schema_name).await;
        DataNode::new(node_name, coordinator, db).await;

        let schema = get_schema(&mut connection, &schema_name).await;
        assert_eq!(schema.as_ref(), schema_name);

        let tables = get_tables(&mut connection, &schema_name).await;
        assert_eq!(tables.len(), 2);
    }
}
