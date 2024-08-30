use super::*;

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
    schema: SchemaName, // Full schema name of data node: data_{node_name}
}

#[async_trait]
impl NodeSchema for DataSchema {
    fn schema(&self) -> SchemaName {
        self.schema.to_owned()
    }

    async fn create_all_tables(&self, connection: &mut PgConnection) {
        self.create_cluster_table(connection).await;
        self.create_record_table(connection).await;
    }
}

impl DataSchema {
    /// Create a new data schema instance based on the node name.
    pub fn new(node: impl Into<SchemaName>) -> Self {
        let schema = format!("data_{}", node.into()).into_boxed_str();
        Self { schema }
    }

    /// Return the name of the table storing vector records.
    pub fn record_table(&self) -> TableName {
        format!("{}.records", self.schema()).into_boxed_str()
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
