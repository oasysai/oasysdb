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
    pub async fn new(database_url: impl Into<DatabaseURL>) -> Self {
        let database_url = database_url.into();
        let mut connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        let schema = CoordinatorSchema::new();
        let parameter_table = schema.parameter_table();
        let params: NodeParameters = sqlx::query_as(&format!(
            "SELECT metric, dimension, density
            FROM {parameter_table}"
        ))
        .fetch_one(&mut connection)
        .await
        .expect("Configure the coordinator node first");

        Self { params, database_url, schema }
    }

    /// Configure the coordinator node with parameters.
    /// - `database_url`: URL to the Postgres database.
    /// - `params`: Coordinator node parameters.
    pub async fn configure(
        database_url: impl Into<DatabaseURL>,
        params: impl Into<NodeParameters>,
    ) {
        let params = params.into();
        let database_url = database_url.into();

        let mut conn = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        let schema = CoordinatorSchema::new();
        schema.create_schema(&mut conn).await;
        schema.create_all_tables(&mut conn).await;

        tracing::info!("database is provisioned for coordinator node");

        let parameter_table = schema.parameter_table();
        sqlx::query(&format!(
            "INSERT INTO {parameter_table} (metric, dimension, density)
            VALUES ($1, $2, $3)
            ON CONFLICT (singleton)
            DO UPDATE SET metric = $1, dimension = $2, density = $3"
        ))
        .bind(params.metric.as_str())
        .bind(params.dimension as i32)
        .bind(params.density as i32)
        .execute(&mut conn)
        .await
        .expect("Failed to configure the coordinator node");
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
    async fn heartbeat(&self, _request: Request<()>) -> ServerResult<()> {
        Ok(Response::new(()))
    }

    async fn register_node(
        &self,
        request: Request<protos::NodeConnection>,
    ) -> ServerResult<protos::NodeParameters> {
        let mut conn = PgConnection::connect(self.database_url().as_ref())
            .await
            .map_err(|_| Status::internal("Failed to connect to Postgres"))?;

        let node = request.into_inner();
        if node.address.parse::<SocketAddr>().is_err() {
            return Err(Status::invalid_argument("Invalid node address"));
        }

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
        let request = Request::new(protos::NodeConnection {
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
        CoordinatorNode::configure(db.to_owned(), params).await;

        let coordinator = CoordinatorNode::new(db).await;
        test_utils::assert_table_count(&mut conn, COORDINATOR_SCHEMA, 4).await;

        Arc::new(coordinator)
    }
}
