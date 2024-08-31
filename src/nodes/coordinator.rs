use super::*;
use protos::coordinator_node_server::CoordinatorNode as ProtoCoordinatorNode;

/// Coordinator node definition.
///
/// Coordinator nodes are responsible for managing the functionality of OasysDB
/// clusters. Data nodes can register with a coordinator node to join a cluster
/// which will horizontally extend OasysDB processing capabilities.
#[derive(Debug)]
pub struct CoordinatorNode {
    params: NodeParameters,
    database_url: DatabaseURL,
    schema: CoordinatorSchema,
}

impl CoordinatorNode {
    /// Create a new coordinator node instance.
    /// - `database_url`: URL to the Postgres database.
    /// - `params`: Parameters to configure new coordinator node.
    pub async fn new(
        database_url: impl Into<DatabaseURL>,
        params: Option<NodeParameters>,
    ) -> Self {
        let database_url = database_url.into();
        let mut connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        let schema = CoordinatorSchema::new();
        schema.create_schema(&mut connection).await;
        schema.create_all_tables(&mut connection).await;

        let parameter_table = schema.parameter_table();
        let existing_params: Option<NodeParameters> = sqlx::query_as(&format!(
            "SELECT metric, dimension, density
            FROM {parameter_table}
            WHERE singleton IS TRUE"
        ))
        .fetch_optional(&mut connection)
        .await
        .expect("Failed to fetch parameters from the database");

        let params = match existing_params {
            Some(params) => params,
            None => {
                let params = params.expect(
                    "Parameters are required for a new node:\n\
                    - dimension: Vector dimensionality\n\
                    - metric: Distance metric (default: euclidean)\n\
                    - density: Number of data per cluster (default: 128)",
                );

                // The parameters are static and should only be inserted once.
                sqlx::query(&format!(
                    "INSERT INTO {parameter_table} (metric, dimension, density)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (singleton) DO NOTHING"
                ))
                .bind(params.metric().as_str())
                .bind(params.dimension() as i32)
                .bind(params.density() as i32)
                .execute(&mut connection)
                .await
                .expect("Failed to insert parameters into the database");

                params
            }
        };

        Self { database_url, schema, params }
    }

    /// Return the coordinator node parameters.
    pub fn params(&self) -> &NodeParameters {
        &self.params
    }

    /// Return the configured database URL.
    pub fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }

    /// Return the schema configured for the coordinator node.
    pub fn schema(&self) -> &CoordinatorSchema {
        &self.schema
    }
}

#[async_trait]
impl ProtoCoordinatorNode for Arc<CoordinatorNode> {
    async fn register_node(
        &self,
        _request: Request<protos::RegisterNodeRequest>,
    ) -> ServerResult<()> {
        Ok(Response::new(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_utils::*;

    const COORDINATOR_SCHEMA: &str = "coordinator";

    #[tokio::test]
    async fn test_coordinator_node_new() {
        let db = database_url();
        let mut connection = PgConnection::connect(&db.to_string())
            .await
            .expect("Failed to connect to Postgres database");

        drop_schema(&mut connection, COORDINATOR_SCHEMA).await;

        let params = NodeParameters::new(DIMENSION);
        CoordinatorNode::new(db, Some(params)).await;

        let tables = get_tables(&mut connection, COORDINATOR_SCHEMA).await;
        assert_eq!(tables.len(), 4);
    }
}
