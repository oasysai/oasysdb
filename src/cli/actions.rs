use super::*;
use oasysdb::nodes::{CoordinatorNode, DataNode};
use oasysdb::postgres::NodeParameters;
use oasysdb::protos::coordinator_node_client::CoordinatorNodeClient;
use oasysdb::protos::coordinator_node_server::CoordinatorNodeServer;
use oasysdb::protos::data_node_server::DataNodeServer;
use oasysdb::protos::NodeConnection;
use reqwest::get;
use std::env;
use std::future::Future;
use tokio::runtime::Runtime;
use tonic::transport::Server;
use tonic::Request;

fn block_on<F: Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create a Tokio runtime");
    rt.block_on(future)
}

fn env_database_url() -> Url {
    env::var("DATABASE_URL")
        .expect("Please set DATABASE_URL environment variable")
        .parse::<Url>()
        .expect("Failed to parse the database URL")
}

// Coordinator action handlers.

pub fn coordinator_handler(args: &ArgMatches) {
    match args.subcommand() {
        Some(("start", _)) => block_on(coordinator_start_handler()),
        Some(("config", args)) => block_on(coordinator_config_handler(args)),
        _ => unreachable!(),
    }
}

async fn coordinator_start_handler() {
    let database_url = env_database_url();
    let node = CoordinatorNode::new(database_url).await;

    let server = CoordinatorNodeServer::new(Arc::new(node));
    let serve = "[::]:2505".parse::<SocketAddr>().unwrap();
    tracing::info!("coordinator server is ready at port {}", serve.port());

    Server::builder()
        .add_service(server)
        .serve(serve)
        .await
        .expect("Failed to start coordinator server");
}

async fn coordinator_config_handler(args: &ArgMatches) {
    let database_url = env_database_url();

    // Unwrap is safe because we have validation and default values in the
    // command configuration in commands.rs file.
    let dimension = args.get_one::<usize>("dim").unwrap();
    let metric = args.get_one::<Metric>("metric").unwrap();
    let density = args.get_one::<usize>("density").unwrap();

    let params = NodeParameters {
        metric: *metric,
        dimension: *dimension,
        density: *density,
    };

    CoordinatorNode::configure(database_url, params).await;
}

// Data action handlers.

pub fn data_handler(args: &ArgMatches) {
    match args.subcommand() {
        Some(("join", args)) => block_on(data_join_handler(args)),
        _ => unreachable!(),
    }
}

async fn data_join_handler(args: &ArgMatches) {
    let database_url = env_database_url();

    // Unwrap is safe because the arguments are validated by clap.
    let name = args.get_one::<String>("name").unwrap().as_str();
    let port = args.get_one::<u16>("port").unwrap();
    let coordinator_addr = format!(
        "http://{}",
        args.get_one::<SocketAddr>("coordinator_addr").unwrap()
    );

    let host = get("https://api.ipify.org")
        .await
        .expect("Failed to retrieve host address")
        .text()
        .await
        .unwrap();

    let request = Request::new(NodeConnection {
        name: name.to_string(),
        address: format!("{host}:{port}"),
    });

    let mut client = CoordinatorNodeClient::connect(coordinator_addr)
        .await
        .expect("Failed to connect to coordinator node");

    let response = client.register_node(request).await.unwrap();
    let params = NodeParameters::from(response.into_inner());
    let node = DataNode::new(name, params, database_url).await;

    let server = DataNodeServer::new(Arc::new(node));
    let serve = format!("[::]:{port}").parse::<SocketAddr>().unwrap();
    tracing::info!("data node server is ready at port {}", serve.port());

    Server::builder()
        .add_service(server)
        .serve(serve)
        .await
        .expect("Failed to start data node server");
}
