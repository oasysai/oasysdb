use super::*;
use protos::coordinator_node_server::CoordinatorNode as ProtoCoordinatorNode;
use tokio::sync::Mutex;

/// Coordinator node definition.
///
/// Coordinator nodes are responsible for managing the functionality of OasysDB
/// clusters. Data nodes can register with a coordinator node to join a cluster
/// which will horizontally extend OasysDB processing capabilities.
#[derive(Debug)]
pub struct CoordinatorNode {
    state: Mutex<CoordinatorState>,
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

        let state_table = schema.state_table();
        let state: CoordinatorState = sqlx::query_as(&format!(
            "SELECT initialized, node_count
            FROM {state_table}"
        ))
        .fetch_one(&mut connection)
        .await
        .unwrap();

        params.trace();
        Self { state: state.into(), params, database_url, schema }
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

        tracing::info!("the database is provisioned for the coordinator");

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
        .expect("Failed to configure the node parameters");

        let state_table = schema.state_table();
        sqlx::query(&format!(
            "INSERT INTO {state_table} (initialized)
            VALUES ($1)
            ON CONFLICT (singleton)
            DO UPDATE SET initialized = $1"
        ))
        .bind(false)
        .execute(&mut conn)
        .await
        .expect("Failed to configure the coordinator state");
    }

    /// Return the parameters of the coordinator node.
    pub fn params(&self) -> &NodeParameters {
        &self.params
    }

    /// Return the schema configuration of the coordinator node.
    pub fn schema(&self) -> &CoordinatorSchema {
        &self.schema
    }
}

impl NodeExt for CoordinatorNode {
    fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }
}

#[async_trait]
impl ProtoCoordinatorNode for Arc<CoordinatorNode> {
    async fn heartbeat(&self, _request: Request<()>) -> ServerResult<()> {
        Ok(Response::new(()))
    }

    async fn register_node(
        &self,
        request: Request<protos::RegisterNodeRequest>,
    ) -> ServerResult<protos::RegisterNodeResponse> {
        let mut conn = self.connect().await?;
        let node = request.into_inner();
        let address = format!("{}:{}", &node.host, &node.port);
        if address.parse::<SocketAddr>().is_err() {
            return Err(Status::invalid_argument("Invalid node address"));
        }

        // TODO: If the cluster is initialized and the node is a new node,
        // transfer some subcluster and records to the new node.

        let connection_table = self.schema.connection_table();
        sqlx::query(&format!(
            "INSERT INTO {connection_table} (name, address)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET address = $2"
        ))
        .bind(&node.name)
        .bind(&address)
        .execute(&mut conn)
        .await
        .map_err(|_| Status::internal("Failed to register node"))?;

        let state_table = self.schema.state_table();
        sqlx::query(&format!(
            "UPDATE {state_table}
            SET node_count = node_count + 1"
        ))
        .execute(&mut conn)
        .await
        .map_err(|_| Status::internal("Failed to update node count"))?;

        tracing::info!("data node \"{}\" has joined the cluster", &node.name);
        Ok(Response::new(self.params().to_owned().into()))
    }

    async fn initialize(
        &self,
        request: Request<protos::InitializeRequest>,
    ) -> ServerResult<()> {
        let mut state = self.state.lock().await;
        if state.initialized {
            let message = "The coordinator node is already initialized";
            return Err(Status::failed_precondition(message));
        }

        let request = request.into_inner();
        let protos::InitializeRequest { records, sampling } = request;

        if sampling <= 0.0 || sampling > 1.0 {
            let message = "Sampling rate must be in the range of 0 and 1";
            return Err(Status::invalid_argument(message));
        }

        let min_samples = self.params().density * 8 * state.node_count;
        let sample_size = (records.len() as f32 * sampling).round() as usize;
        if sample_size < min_samples {
            let message = format!("The minimum sample size is {min_samples}");
            return Err(Status::invalid_argument(message));
        }

        let mut conn = self.connect().await?;

        let state_table = self.schema.state_table();
        sqlx::query(&format!(
            "UPDATE {state_table}
            SET initialized = $1"
        ))
        .bind(true)
        .execute(&mut conn)
        .await
        .map_err(|_| Status::internal("Failed to update the state"))?;

        state.initialized = true;
        Ok(Response::new(()))
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
        let request = Request::new(protos::RegisterNodeRequest {
            name: "c12eb363".to_string(),
            host: "0.0.0.0".to_string(),
            port: 2510,
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
        test_utils::assert_table_count(&mut conn, COORDINATOR_SCHEMA, 5).await;

        Arc::new(coordinator)
    }
}
