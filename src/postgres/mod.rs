mod coordinator;
mod data;

// Re-export types from submodules.
pub use coordinator::*;
pub use data::*;

// Import common dependencies below.
use async_trait::async_trait;
use sqlx::PgConnection;

type SchemaName = Box<str>;
type TableName = Box<str>;

/// Trait of a node schema in Postgres database.
///
/// The schema of a coordinator node and a data node are mostly different. This
/// trait defines the common methods for both types of nodes.
#[async_trait]
pub trait NodeSchema {
    /// Return the schema name of the node.
    fn schema(&self) -> SchemaName;

    /// Return the table name storing cluster data.
    fn cluster_table(&self) -> TableName {
        format!("{}.clusters", self.schema()).into_boxed_str()
    }

    /// Create a new schema belonging to a node in the database.
    async fn create_schema(&self, connection: &mut PgConnection) {
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", self.schema()))
            .execute(connection)
            .await
            .expect("Failed to create the schema");
    }

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
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use sqlx::Row;
    use url::Url as DatabaseURL;

    /// Return a database URL for testing purposes.
    pub fn database_url() -> DatabaseURL {
        let url = "postgres://postgres:password@0.0.0.0:5432/postgres";
        DatabaseURL::parse(url).unwrap()
    }

    pub async fn drop_schema(
        connection: &mut PgConnection,
        schema: impl AsRef<str>,
    ) {
        sqlx::query(&format!(
            "DROP SCHEMA IF EXISTS {} CASCADE",
            schema.as_ref()
        ))
        .execute(connection)
        .await
        .expect("Failed to drop the schema");
    }

    pub async fn get_tables(
        connection: &mut PgConnection,
        schema: impl AsRef<str>,
    ) -> Vec<String> {
        sqlx::query(
            "SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = $1
            AND table_type = 'BASE TABLE'",
        )
        .bind(schema.as_ref())
        .fetch_all(connection)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>(0))
        .collect()
    }
}
