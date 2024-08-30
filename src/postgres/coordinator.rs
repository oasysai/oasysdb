use super::*;

/// Database schema for a coordinator node.
///
/// The coordinator schema is used to isolate and manage the tables dedicated
/// to the coordinator node. By default, the schema name is coordinator.
///
/// The schema contains the following tables:
/// - clusters: Storing cluster information.
/// - connections: Storing data node connections.
/// - subclusters: Storing sub-cluster information.
///
/// P.S. Sub-clusters are clusters from the data nodes.
#[derive(Debug)]
pub struct CoordinatorSchema {
    schema: SchemaName,
}

impl Default for CoordinatorSchema {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeSchema for CoordinatorSchema {
    fn schema(&self) -> SchemaName {
        self.schema.to_owned()
    }
}

impl CoordinatorSchema {
    /// Create a new instance of the coordinator schema.
    pub fn new() -> Self {
        Self { schema: "coordinator".into() }
    }

    /// Return the name of the table storing data node connections.
    pub fn connection_table(&self) -> TableName {
        format!("{}.connections", self.schema()).into_boxed_str()
    }

    /// Return the name of the table storing sub-clusters.
    pub fn subcluster_table(&self) -> TableName {
        format!("{}.subclusters", self.schema()).into_boxed_str()
    }

    /// Create a table to track data node connections.
    ///
    /// Columns:
    /// - name: Unique name of the data node.
    /// - address: Network address to connect to the data node.
    pub async fn create_connection_table(&self, connection: &mut PgConnection) {
        let table = self.connection_table();
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (
                name TEXT PRIMARY KEY,
                address TEXT NOT NULL
            )"
        ))
        .execute(connection)
        .await
        .expect("Failed to create the connection table");
    }

    /// Create a table to store clusters from data nodes.
    ///
    /// Columns:
    /// - id: Unique ID of the data node cluster.
    /// - connection_name: Data node name of the sub-cluster.
    /// - cluster_id: Cluster ID assigned for the sub-cluster.
    /// - centroid: Centroid vector as a byte array.
    pub async fn create_subcluster_table(&self, connection: &mut PgConnection) {
        let subcluster_table = self.subcluster_table();
        let connection_table = self.connection_table();
        let cluster_table = self.cluster_table();

        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {subcluster_table} (
                id UUID PRIMARY KEY,
                connection_name TEXT NOT NULL REFERENCES {connection_table} (name),
                cluster_id UUID NOT NULL REFERENCES {cluster_table} (id),
                centroid BYTEA NOT NULL
            )"
        ))
        .execute(connection)
        .await
        .expect("Failed to create the subcluster table");
    }
}
