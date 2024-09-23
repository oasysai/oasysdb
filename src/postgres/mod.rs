mod rows;
mod schemas;

// Re-export types from submodules.
pub use rows::*;
pub use schemas::*;

// Import common dependencies below.
use crate::types::Metric;
use async_trait::async_trait;
use sqlx::PgConnection;

type SchemaName = Box<str>;
type TableName = Box<str>;

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use sqlx::Row;
    use url::Url as DatabaseURL;

    /// Return the default node parameters for testing purposes.
    /// - metric: Euclidean
    /// - dimension: 768
    /// - density: 256
    pub fn node_parameters() -> NodeParameters {
        NodeParameters {
            metric: Metric::Euclidean,
            dimension: 128,
            density: 256,
        }
    }

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

    /// Assert the number of tables in a schema to the value.
    pub async fn assert_table_count(
        connection: &mut PgConnection,
        schema: impl AsRef<str>,
        expected: usize,
    ) {
        let count = sqlx::query(
            "SELECT COUNT(*)
            FROM information_schema.tables
            WHERE table_schema = $1
            AND table_type = 'BASE TABLE'",
        )
        .bind(schema.as_ref())
        .fetch_one(connection)
        .await
        .unwrap()
        .get::<i64, usize>(0) as usize;

        assert_eq!(count, expected);
    }
}
