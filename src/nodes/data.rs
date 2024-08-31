use super::*;
use protos::data_node_server::DataNode as ProtoDataNode;
use regex::Regex;

type NodeName = Box<str>;

// TODO: Add parameters to the data node.

/// Data node server definition.
///
/// Data nodes are responsible for managing the records and data processing
/// capabilities of OasysDB clusters. Various data nodes can register with a
/// coordinator node.
#[derive(Debug)]
pub struct DataNode {
    name: NodeName,
    coordinator_url: CoordinatorURL,
    database_url: DatabaseURL,
    schema: DataSchema,
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

        let schema = DataSchema::new(name.as_ref());
        schema.create_schema(&mut connection).await;
        schema.create_all_tables(&mut connection).await;

        // Register with the coordinator.

        Self { name, coordinator_url, database_url, schema }
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

    /// Return the schema configured for the data node.
    pub fn schema(&self) -> &DataSchema {
        &self.schema
    }
}

#[async_trait]
impl ProtoDataNode for Arc<DataNode> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_utils::*;

    const DATA_SCHEMA: &str = "data_";

    fn coordinator_url() -> CoordinatorURL {
        "0.0.0.0:2505".parse::<SocketAddr>().unwrap()
    }

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

        let tables = get_tables(&mut connection, &schema_name).await;
        assert_eq!(tables.len(), 2);
    }
}
