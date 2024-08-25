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
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ?")
        .bind(schema.as_ref())
        .execute(connection)
        .await
        .expect("Failed to create the schema");
}

/// Create a table to store cluster data.
async fn create_cluster_table(
    connection: &mut PgConnection,
    schema: impl AsRef<str>,
) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ? (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            centroid BYTEA NOT NULL
        )",
    )
    .bind(format!("{}.{CLUSTER_TABLE}", schema.as_ref()))
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
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ? (
            name TEXT PRIMARY KEY,
            address TEXT NOT NULL,
        )",
    )
    .bind(format!("{COORDINATOR_SCHEMA}.{CONNECTION_TABLE}"))
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
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ? (
            id UUID PRIMARY KEY,
            connection_name TEXT NOT NULL REFERENCES ? (name),
            cluster_id UUID NOT NULL REFERENCES ? (id),
            centroid BYTEA NOT NULL,
        )",
    )
    .bind(format!("{COORDINATOR_SCHEMA}.{SUBCLUSTER_TABLE}"))
    .bind(format!("{COORDINATOR_SCHEMA}.{CONNECTION_TABLE}"))
    .bind(format!("{COORDINATOR_SCHEMA}.{CLUSTER_TABLE}"))
    .execute(connection)
    .await
    .expect("Failed to create the subcentroid table");
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
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ? (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            cluster_id UUID NOT NULL REFERENCES ? (id),
            vector BYTEA NOT NULL,
            data JSONB
        )",
    )
    .bind(format!("{}.{RECORD_TABLE}", schema.as_ref()))
    .bind(format!("{}.{CLUSTER_TABLE}", schema.as_ref()))
    .execute(connection)
    .await
    .expect("Failed to create the record table");
}
