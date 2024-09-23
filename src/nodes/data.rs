use super::*;
use crate::protod;
use protod::data_node_server::DataNode as ProtoDataNode;
use regex::Regex;

type NodeName = Box<str>;

/// Data node server definition.
///
/// Data nodes are responsible for managing the records and data processing
/// capabilities of OasysDB clusters. Various data nodes can register with a
/// coordinator node.
#[derive(Debug)]
pub struct DataNode {
    name: NodeName,
    params: NodeParameters,
    database_url: DatabaseURL,
    schema: DataSchema,
}

impl DataNode {
    /// Create a new data node instance.
    pub async fn new(
        name: impl Into<NodeName>,
        params: impl Into<NodeParameters>,
        database_url: impl Into<DatabaseURL>,
    ) -> Self {
        let name = name.into();
        let params = params.into();
        let database_url = database_url.into();

        // Validate node name: lowercase, alphanumeric, and underscores only.
        if !Regex::new("^[a-z0-9_]+$").unwrap().is_match(name.as_ref()) {
            let action = "Use lowercase letters, numbers, and underscores.";
            panic!("Invalid node name: {action}");
        }

        let mut connection = PgConnection::connect(database_url.as_ref())
            .await
            .expect("Failed to connect to Postgres database");

        let schema = DataSchema::new(name.as_ref());
        if !schema.exists(&mut connection).await {
            schema.create(&mut connection).await;
            schema.create_all_tables(&mut connection).await;
            tracing::info!("database is provisioned for data node: {name}");
        }

        Self { name, params, database_url, schema }
    }

    /// Return the name configured for this data node.
    pub fn name(&self) -> &NodeName {
        &self.name
    }

    /// Return the parameters configuration of the data node.
    pub fn params(&self) -> &NodeParameters {
        &self.params
    }
}

impl NodeExt for DataNode {
    fn database_url(&self) -> &DatabaseURL {
        &self.database_url
    }

    fn schema(&self) -> &impl NodeSchema {
        &self.schema
    }
}

#[async_trait]
impl ProtoDataNode for Arc<DataNode> {
    async fn heartbeat(
        &self,
        _request: Request<protod::HeartbeatRequest>,
    ) -> ServerResult<protod::HeartbeatResponse> {
        Ok(Response::new(protod::HeartbeatResponse {}))
    }

    async fn insert_cluster(
        &self,
        request: Request<protod::InsertClusterRequest>,
    ) -> ServerResult<protod::InsertClusterResponse> {
        let request = request.into_inner();
        let centroid: Vector = request.centroid.into();

        let mut conn = self.connect().await?;
        let id = self._insert_cluster(&mut conn, &centroid).await?;

        Ok(Response::new(protod::InsertClusterResponse { id: id.to_string() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::test_utils;
    use sqlx::Row;

    const DATA_SCHEMA: &str = "odb_node_";

    #[tokio::test]
    async fn test_data_node_new() {
        let node_name = "49babacf";
        data_node_mock_server(node_name).await;
    }

    #[tokio::test]
    async fn test_data_node_insert_cluster() {
        let node_name = "d4835657";
        let node = data_node_mock_server(node_name).await;

        let dimension = node.params().dimension;
        for _ in 0..10 {
            let centroid = vec![rand::random::<f32>(); dimension];
            let request = protod::InsertClusterRequest { centroid };
            node.insert_cluster(Request::new(request)).await.unwrap();
        }

        let mut conn = node.connect().await.unwrap();
        let cluster_table = node.schema().cluster_table();
        let query = format!("SELECT COUNT(*) FROM {cluster_table}");
        let count = sqlx::query(&query)
            .fetch_one(&mut conn)
            .await
            .unwrap()
            .get::<i64, _>(0);

        assert_eq!(count, 10);
    }

    async fn data_node_mock_server(name: impl Into<NodeName>) -> Arc<DataNode> {
        let name = name.into();
        let schema_name = format!("{DATA_SCHEMA}{name}");

        let params = test_utils::node_parameters();
        let db = test_utils::database_url();

        let mut conn = PgConnection::connect(&db.to_string())
            .await
            .expect("Failed to connect to Postgres database");

        test_utils::drop_schema(&mut conn, &schema_name).await;
        let node = DataNode::new(name, params, db).await;
        test_utils::assert_table_count(&mut conn, &schema_name, 2).await;

        Arc::new(node)
    }
}
