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
const DATA_SCHEMA: &str = "data";

// Constants for database table names.
const CENTROID_TABLE: &str = "centroids";
const CLUSTER_TABLE: &str = "clusters";
const RECORD_TABLE: &str = "records";
const CONNECTION_TABLE: &str = "connections";
const SUBCENTROID_TABLE: &str = "subcentroids";

/// Create a table to track data nodes connected to the coordinator.
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

/// Create a table for the coordinator to store centroids from data nodes.
async fn create_coordinator_subcentroid_table(connection: &mut PgConnection) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ? (
            id UUID PRIMARY KEY,
            connection_name TEXT NOT NULL REFERENCES ? (name),
            vector BYTEA NOT NULL,
        )",
    )
    .bind(format!("{COORDINATOR_SCHEMA}.{SUBCENTROID_TABLE}"))
    .bind(format!("{COORDINATOR_SCHEMA}.{CONNECTION_TABLE}"))
    .execute(connection)
    .await
    .expect("Failed to create the subcentroid table");
}

/// Create a table to track of which subcentroids belong to which centroids.
async fn create_coordinator_cluster_table(connection: &mut PgConnection) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ? (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            centroid_id UUID NOT NULL REFERENCES ? (id),
            subcentroid_id UUID NOT NULL REFERENCES ? (id),
        )",
    )
    .bind(format!("{COORDINATOR_SCHEMA}.{CLUSTER_TABLE}"))
    .bind(format!("{COORDINATOR_SCHEMA}.{CENTROID_TABLE}"))
    .bind(format!("{COORDINATOR_SCHEMA}.{SUBCENTROID_TABLE}"))
    .execute(connection)
    .await
    .expect("Failed to create the cluster table");
}

// Common utility functions.

async fn create_schema(connection: &mut PgConnection, schema: impl AsRef<str>) {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ?")
        .bind(schema.as_ref())
        .execute(connection)
        .await
        .expect("Failed to create the schema");
}

async fn create_centroid_table(
    connection: &mut PgConnection,
    schema: impl AsRef<str>,
) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ? (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            vector BYTEA NOT NULL
        )",
    )
    .bind(format!("{}.{CENTROID_TABLE}", schema.as_ref()))
    .execute(connection)
    .await
    .expect("Failed to create centroid table");
}
