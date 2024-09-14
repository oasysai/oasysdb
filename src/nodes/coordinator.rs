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
        if !schema.exists(&mut conn).await {
            schema.create(&mut conn).await;
            schema.create_all_tables(&mut conn).await;
            tracing::info!("the database is provisioned for the coordinator");
        }

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

        tracing::info!("the coordinator node is configured successfully");
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
        let node = request.into_inner();
        let address = format!("{}:{}", &node.host, &node.port);
        if address.parse::<SocketAddr>().is_err() {
            return Err(Status::invalid_argument("Invalid node address"));
        }

        let mut conn = self.connect().await?;
        let connection_table = self.schema.connection_table();
        let existing_node: Option<NodeConnection> = sqlx::query_as(&format!(
            "SELECT name, address
            FROM {connection_table}
            WHERE name = $1"
        ))
        .bind(&node.name)
        .fetch_optional(&mut conn)
        .await
        .map_err(|_| Status::internal("Failed to retrieve a node detail"))?;

        match existing_node {
            Some(_) => self.register_existing_node(&mut conn, &node).await?,
            None => {
                self.register_new_node(&mut conn, &node).await?;
                self.increment_node_count(&mut conn).await?

                // TODO: If the cluster is initialized transfer some subcluster
                // and records to the new node to balance the load.
            }
        };

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

impl CoordinatorNode {
    async fn increment_node_count(
        &self,
        conn: &mut PgConnection,
    ) -> Result<(), Status> {
        let state_table = self.schema.state_table();
        sqlx::query(&format!(
            "UPDATE {state_table}
            SET node_count = node_count + 1"
        ))
        .execute(conn)
        .await
        .map_err(|_| Status::internal("Failed to update node count"))?;

        let mut state = self.state.lock().await;
        state.node_count += 1;
        Ok(())
    }

    async fn register_existing_node(
        &self,
        conn: &mut PgConnection,
        node: &protos::RegisterNodeRequest,
    ) -> Result<(), Status> {
        let connection_table = self.schema.connection_table();
        let address = format!("{}:{}", &node.host, &node.port);

        sqlx::query(&format!(
            "UPDATE {connection_table}
            SET address = $1
            WHERE name = $2"
        ))
        .bind(&address)
        .bind(&node.name)
        .execute(conn)
        .await
        .map_err(|_| Status::internal("Failed to update existing node"))?;

        tracing::info!("data node \"{}\" rejoins the cluster", &node.name);
        Ok(())
    }

    async fn register_new_node(
        &self,
        conn: &mut PgConnection,
        node: &protos::RegisterNodeRequest,
    ) -> Result<(), Status> {
        let connection_table = self.schema.connection_table();
        let address = format!("{}:{}", &node.host, &node.port);

        sqlx::query(&format!(
            "INSERT INTO {connection_table} (name, address)
            VALUES ($1, $2)"
        ))
        .bind(&node.name)
        .bind(&address)
        .execute(conn)
        .await
        .map_err(|_| Status::internal("Failed to register new node"))?;

        tracing::info!("registered a new data node: {}", &node.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_utils;

    const COORDINATOR_SCHEMA: &str = "odb_coordinator";

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
