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
                    - metric: Distance metric (Default: Euclidean)\n\
                    - density: Number of data per cluster (Default: 128)",
                );

                // The parameters are static and should only be inserted once.
                sqlx::query(&format!(
                    "INSERT INTO {parameter_table} (metric, dimension, density)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (singleton) DO NOTHING"
                ))
                .bind(params.metric.as_str())
                .bind(params.dimension as i32)
                .bind(params.density as i32)
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
        request: Request<protos::DataNodeConnection>,
    ) -> ServerResult<protos::NodeParameters> {
        let mut conn = PgConnection::connect(self.database_url().as_ref())
            .await
            .map_err(|_| Status::internal("Failed to connect to Postgres"))?;

        let node = request.into_inner();
        let connection_table = self.schema().connection_table();
        sqlx::query(&format!(
            "INSERT INTO {connection_table} (name, address)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET address = $2"
        ))
        .bind(&node.name)
        .bind(&node.address)
        .execute(&mut conn)
        .await
        .map_err(|_| Status::internal("Failed to register node"))?;

        tracing::info!("data node \"{}\" has been registered", &node.name);
        Ok(Response::new(self.params().to_owned().into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_utils;

    const COORDINATOR_SCHEMA: &str = "coordinator";

    #[tokio::test]
    async fn test_coordinator_node_new() {
        coordinator_node_mock_server().await;
    }

    #[tokio::test]
    async fn test_coordinator_node_register_node() {
        let coordinator = coordinator_node_mock_server().await;
        let request = Request::new(protos::DataNodeConnection {
            name: "c12eb363".to_string(),
            address: "0.0.0.0:2510".to_string(),
        });

        let response = coordinator.register_node(request).await.unwrap();
        let params = response.into_inner();
        assert_eq!(params.dimension, 768);
    }

    async fn coordinator_node_mock_server() -> Arc<CoordinatorNode> {
        let params = test_utils::node_parameters();
        let db = test_utils::database_url();

        let mut conn = PgConnection::connect(&db.to_string()).await.unwrap();
        test_utils::drop_schema(&mut conn, COORDINATOR_SCHEMA).await;

        let coordinator = CoordinatorNode::new(db, Some(params)).await;
        test_utils::assert_table_count(&mut conn, COORDINATOR_SCHEMA, 4).await;

        Arc::new(coordinator)
    }
}
