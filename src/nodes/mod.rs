mod coordinator;
mod data;

// Re-export types from submodules.
pub use coordinator::*;
pub use data::*;

type DatabaseURL = Box<str>;
type ServerResult<T> = StandardResult<Response<T>, Status>;

// Import common modules below.
use crate::protos;
use sqlx::{Connection, PgConnection};
use std::result::Result as StandardResult;
use std::sync::Arc;
use tonic::{async_trait, Request, Response, Status};

// Constants for database schema names.
const COORDINATOR_SCHEMA: &str = "coordinator";
/// Append the data node name for the final schema name.
const DATA_SCHEMA: &str = "data_";

// Constants for database table names.
const CONNECTION_TABLE: &str = "connections";
const CLUSTER_TABLE: &str = "clusters";
const RECORD_TABLE: &str = "records";
const SUBCLUSTER_TABLE: &str = "subclusters";

/// Create a new schema with the given name in the database.
async fn create_schema(connection: &mut PgConnection, schema: impl AsRef<str>) {
    sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", schema.as_ref()))
        .execute(connection)
        .await
        .expect("Failed to create the schema");
}

/// Create a table to store cluster data.
async fn create_cluster_table(
    connection: &mut PgConnection,
    schema: impl AsRef<str>,
) {
    let schema = schema.as_ref();
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {schema}.{CLUSTER_TABLE} (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            centroid BYTEA NOT NULL
        )"
    ))
    .execute(connection)
    .await
    .expect("Failed to create cluster table");
}

/// Create a table to track data node connections.
///
/// Columns:
/// - name: Unique name of the data node.
/// - address: Network address to connect to the data node.
async fn create_coordinator_connection_table(connection: &mut PgConnection) {
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {COORDINATOR_SCHEMA}.{CONNECTION_TABLE} (
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
async fn create_coordinator_subcluster_table(connection: &mut PgConnection) {
    let subcluster_table = format!("{COORDINATOR_SCHEMA}.{SUBCLUSTER_TABLE}");
    let connection_table = format!("{COORDINATOR_SCHEMA}.{CONNECTION_TABLE}");
    let cluster_table = format!("{COORDINATOR_SCHEMA}.{CLUSTER_TABLE}");

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

/// Create a table to store vector records.
///
/// Columns:
/// - id: Record ID.
/// - cluster_id: Cluster ID assigned for the record.
/// - vector: Record vector as a byte array.
/// - data: Additional metadata as a JSON object.
async fn create_data_record_table(
    connection: &mut PgConnection,
    schema: impl AsRef<str>,
) {
    let schema = schema.as_ref();
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS {schema}.{RECORD_TABLE} (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            cluster_id UUID NOT NULL REFERENCES {schema}.{CLUSTER_TABLE} (id),
            vector BYTEA NOT NULL,
            data JSONB
        )"
    ))
    .execute(connection)
    .await
    .expect("Failed to create the data record table");
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    pub const DB: &str = "postgres://postgres:password@0.0.0.0:5432/postgres";

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

    pub async fn get_schema(
        connection: &mut PgConnection,
        schema: impl AsRef<str>,
    ) -> Box<str> {
        sqlx::query(
            "SELECT schema_name
            FROM information_schema.schemata
            WHERE schema_name = $1",
        )
        .bind(schema.as_ref())
        .fetch_one(connection)
        .await
        .unwrap()
        .get::<String, _>(0)
        .into_boxed_str()
    }

    pub async fn get_tables(
        connection: &mut PgConnection,
        schema: impl AsRef<str>,
    ) -> Box<[Box<str>]> {
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
        .map(|row| row.get::<String, _>(0).into_boxed_str())
        .collect()
    }
}
