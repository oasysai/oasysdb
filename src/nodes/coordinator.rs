use super::*;
use crate::protoc;
use futures::StreamExt;
use protoc::coordinator_node_server::CoordinatorNode as ProtoCoordinatorNode;

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

        params.trace();
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

        tracing::info!("the coordinator node is configured successfully");
    }

    /// Return the parameters of the coordinator node.
    pub fn params(&self) -> &NodeParameters {
        &self.params
    }
}

impl NodeExt for CoordinatorNode {
    fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }

    fn schema(&self) -> &impl NodeSchema {
        &self.schema
    }
}

#[async_trait]
impl ProtoCoordinatorNode for Arc<CoordinatorNode> {
    async fn heartbeat(
        &self,
        _request: Request<protoc::HeartbeatRequest>,
    ) -> ServerResult<protoc::HeartbeatResponse> {
        // TODO: Check the heartbeat of all the data nodes in the cluster.
        Ok(Response::new(protoc::HeartbeatResponse {}))
    }

    async fn register_node(
        &self,
        request: Request<protoc::RegisterNodeRequest>,
    ) -> ServerResult<protoc::RegisterNodeResponse> {
        let connection = request.into_inner().connection;
        let node = match connection {
            Some(node) => node,
            None => {
                let message = "Node connection is required to register a node";
                return Err(Status::invalid_argument(message));
            }
        };

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

                // TODO: If the cluster is initialized transfer some subcluster
                // and records to the new node to balance the load.
            }
        };

        Ok(Response::new(protoc::RegisterNodeResponse {
            parameters: Some(self.params().to_owned().into()),
        }))
    }

    async fn insert(
        &self,
        request: Request<protoc::InsertRequest>,
    ) -> ServerResult<protoc::InsertResponse> {
        let request = request.into_inner();
        let record = match request.record {
            Some(record) => record,
            None => {
                let message = "Record is required to insert into the database";
                return Err(Status::invalid_argument(message));
            }
        };

        if record.vector.len() != self.params.dimension {
            return Err(Status::invalid_argument(format!(
                "Vector dimension mismatch, expected {}, found {}",
                self.params.dimension,
                record.vector.len()
            )));
        }

        let mut conn = self.connect().await?;
        let vector: Vector = record.vector.into();
        let cluster = self.find_nearest_cluster(&mut conn, &vector).await?;

        match cluster {
            Some(_cluster) => {}
            None => {
                let _cid = self.insert_cluster(&mut conn, &vector).await?;
            }
        }

        Ok(Response::new(protoc::InsertResponse { id: "1".to_string() }))
    }
}

impl CoordinatorNode {
    async fn register_existing_node(
        &self,
        conn: &mut PgConnection,
        node: &protoc::NodeConnection,
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
        node: &protoc::NodeConnection,
    ) -> Result<(), Status> {
        let connection_table = self.schema.connection_table();
        let address = format!("{}:{}", node.host, node.port);

        sqlx::query(&format!(
            "INSERT INTO {connection_table} (name, address)
            VALUES ($1, $2)"
        ))
        .bind(&node.name)
        .bind(address)
        .execute(conn)
        .await
        .map_err(|_| Status::internal("Failed to register new node"))?;

        tracing::info!("registered a new data node: {}", &node.name);
        Ok(())
    }

    async fn find_nearest_cluster(
        &self,
        conn: &mut PgConnection,
        vector: &Vector,
    ) -> Result<Option<Cluster>, Status> {
        let cluster_table = self.schema.cluster_table();
        let query = format!(
            "SELECT id, centroid, count
            FROM {cluster_table}"
        );

        let mut rows = sqlx::query(&query).fetch(conn);
        let mut nearest_cluster: Option<Cluster> = None;
        let mut min_distance = f32::MAX;

        while let Some(row) = rows.next().await {
            let row = row.map_err(|e| {
                let message = format!("Failed to retrieve cluster: {e}");
                Status::internal(message)
            })?;

            let cluster = Cluster::from_row(&row).map_err(|e| {
                let message = format!("Failed to downcast cluster type: {e}");
                Status::internal(message)
            })?;

            let metric = self.params().metric;
            let distance = metric.distance(vector, &cluster.centroid);
            if distance < min_distance {
                min_distance = distance;
                nearest_cluster = Some(cluster);
            }
        }

        Ok(nearest_cluster)
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
        let request = Request::new(protoc::RegisterNodeRequest {
            connection: Some(protoc::NodeConnection {
                name: "c12eb363".to_string(),
                host: "0.0.0.0".to_string(),
                port: 2510,
            }),
        });

        let response = coordinator.register_node(request).await.unwrap();
        let params = response.into_inner().parameters.unwrap();
        assert_eq!(params.dimension, 128);
    }

    #[tokio::test]
    async fn test_coordinator_node_find_nearest_cluster() {
        let coordinator = coordinator_node_mock_server().await;
        let dimension = coordinator.params().dimension;

        let db = test_utils::database_url();
        let mut conn = PgConnection::connect(&db.to_string()).await.unwrap();

        let mut ids = Vec::new();
        for i in 1..11 {
            let centroid = vec![i as f32; dimension];
            let id = coordinator
                .insert_cluster(&mut conn, &centroid.into())
                .await
                .unwrap();

            ids.push(id);
        }

        let query = vec![1.0; dimension];
        let cluster = coordinator
            .find_nearest_cluster(&mut conn, &query.into())
            .await
            .unwrap();

        assert_eq!(cluster.unwrap().id, ids[0]);
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
