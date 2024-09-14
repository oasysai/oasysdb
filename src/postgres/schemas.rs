use super::*;

/// Trait of a node schema in Postgres database.
///
/// The schema of a coordinator node and a data node are mostly different. This
/// trait defines the common methods for both types of nodes.
#[async_trait]
pub trait NodeSchema {
    /// Return the schema name of the node.
    fn name(&self) -> SchemaName;

    /// Return the table name storing cluster data.
    fn cluster_table(&self) -> TableName {
        format!("{}.clusters", self.name()).into_boxed_str()
    }

    /// Create a new schema belonging to a node in the database.
    async fn create(&self, connection: &mut PgConnection) {
        tracing::info!("creating a database schema: {}", self.name());
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", self.name()))
            .execute(connection)
            .await
            .expect("Failed to create the schema");
    }

    /// Create all tables required by the node.
    async fn create_all_tables(&self, connection: &mut PgConnection);

    /// Create a table to store cluster data.
    ///
    /// Columns:
    /// - id: Cluster ID.
    /// - centroid: Centroid vector of the cluster.
    async fn create_cluster_table(&self, connection: &mut PgConnection) {
        let table = self.cluster_table();
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                centroid BYTEA NOT NULL
            )"
        ))
        .execute(connection)
        .await
        .expect("Failed to create cluster table");
    }

    /// Return true if the schema exists in the database.
    async fn exists(&self, connection: &mut PgConnection) -> bool {
        let schema_name = self.name();
        let row = sqlx::query(&format!(
            "SELECT schema_name FROM information_schema.schemata
            WHERE schema_name = '{schema_name}'"
        ))
        .fetch_optional(connection)
        .await
        .expect("Failed to check if schema exists");

        row.is_some()
    }
}

/// Database schema for a coordinator node.
///
/// The coordinator schema is used to isolate and manage the tables dedicated
/// to the coordinator node. By default, the schema name is coordinator.
///
/// The schema contains the following tables:
/// - states: Storing coordinator node states.
/// - parameters: Storing node parameters.
/// - clusters: Storing cluster information.
/// - connections: Storing data node connections.
/// - subclusters: Storing sub-cluster information.
///
/// P.S. Sub-clusters are clusters from the data nodes.
#[derive(Debug)]
pub struct CoordinatorSchema {
    name: SchemaName,
}

impl Default for CoordinatorSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeSchema for CoordinatorSchema {
    fn name(&self) -> SchemaName {
        self.name.to_owned()
    }

    async fn create_all_tables(&self, connection: &mut PgConnection) {
        tracing::info!("creating tables for the coordinator node");

        self.create_state_table(connection).await;
        self.create_parameter_table(connection).await;

        self.create_cluster_table(connection).await;
        self.create_connection_table(connection).await;
        self.create_subcluster_table(connection).await;
    }
}

impl CoordinatorSchema {
    /// Create a new instance of the coordinator schema.
    pub fn new() -> Self {
        Self { name: "odb_coordinator".into() }
    }

    /// Return the table name storing the node states.
    pub fn state_table(&self) -> TableName {
        format!("{}.states", self.name()).into_boxed_str()
    }

    /// Return the table name storing the node parameters.
    pub fn parameter_table(&self) -> TableName {
        format!("{}.parameters", self.name()).into_boxed_str()
    }

    /// Return the name of the table storing data node connections.
    pub fn connection_table(&self) -> TableName {
        format!("{}.connections", self.name()).into_boxed_str()
    }

    /// Return the name of the table storing sub-clusters.
    pub fn subcluster_table(&self) -> TableName {
        format!("{}.subclusters", self.name()).into_boxed_str()
    }

    /// Create a table to store node states.
    ///
    /// Columns:
    /// - initialized: Whether the node is initialized.
    pub async fn create_state_table(&self, connection: &mut PgConnection) {
        let table = self.state_table();
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (
                singleton BOOLEAN PRIMARY KEY DEFAULT true,
                initialized BOOLEAN NOT NULL,
                node_count INTEGER NOT NULL DEFAULT 0,

                CONSTRAINT single_row CHECK (singleton)
            )"
        ))
        .execute(connection)
        .await
        .expect("Failed to create the state table");
    }

    /// Create a table to store node parameters.
    ///
    /// Columns:
    /// - metric: Metric used to calculate distance.
    /// - dimension: Vector dimension.
    /// - density: Number of records in each cluster.
    pub async fn create_parameter_table(&self, connection: &mut PgConnection) {
        let table = self.parameter_table();
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (
                singleton BOOLEAN PRIMARY KEY DEFAULT true,
                metric TEXT NOT NULL,
                dimension INTEGER NOT NULL,
                density INTEGER NOT NULL,

                CONSTRAINT single_row CHECK (singleton),
                CONSTRAINT valid_dimension CHECK (dimension > 0),
                CONSTRAINT valid_density CHECK (density > 0),
                CONSTRAINT valid_metric CHECK (
                    metric IN (
                        'euclidean',
                        'cosine'
                    )
                )
            )"
        ))
        .execute(connection)
        .await
        .expect("Failed to create the parameter table");
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

/// Database schema for a data node.
///
/// Data node schema name is dynamically generated based on the node name
/// which is user-defined.
///
/// Tables:
/// - clusters: Storing cluster information.
/// - records: Storing vector records.
#[derive(Debug)]
pub struct DataSchema {
    name: SchemaName, // Full schema name of data node: odb_node_{node_name}
}

#[async_trait]
impl NodeSchema for DataSchema {
    fn name(&self) -> SchemaName {
        self.name.to_owned()
    }

    async fn create_all_tables(&self, connection: &mut PgConnection) {
        tracing::info!("creating tables for the data node.");
        self.create_cluster_table(connection).await;
        self.create_record_table(connection).await;
    }
}

impl DataSchema {
    /// Create a new data schema based on the node name.
    pub fn new(node: impl Into<SchemaName>) -> Self {
        let name = format!("odb_node_{}", node.into()).into_boxed_str();
        Self { name }
    }

    /// Return the name of the table storing vector records.
    pub fn record_table(&self) -> TableName {
        format!("{}.records", self.name()).into_boxed_str()
    }

    /// Create a table to store vector records.
    ///
    /// Columns:
    /// - id: Record ID.
    /// - cluster_id: Cluster ID assigned for the record.
    /// - vector: Record vector as a byte array.
    /// - data: Additional metadata as a JSON object.
    pub async fn create_record_table(&self, connection: &mut PgConnection) {
        let record_table = self.record_table();
        let cluster_table = self.cluster_table();

        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {record_table} (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                cluster_id UUID NOT NULL REFERENCES {cluster_table} (id),
                vector BYTEA NOT NULL,
                data JSONB
            )"
        ))
        .execute(connection)
        .await
        .expect("Failed to create the data record table");
    }
}
