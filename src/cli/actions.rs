use super::*;
use oasysdb::nodes::{CoordinatorNode, DataNode};
use oasysdb::protos::coordinator_node_server::CoordinatorNodeServer;
use oasysdb::protos::data_node_server::DataNodeServer;
use std::future::Future;
use std::net::SocketAddr;
use tokio::runtime::Runtime;
use tonic::transport::Server;

fn block_on<F: Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create a Tokio runtime.");
    rt.block_on(future)
}

// Coordinator action handlers.

pub fn coordinator_handler(args: &ArgMatches) {
    match args.subcommand() {
        Some(("start", args)) => coordinator_start_handler(args),
        _ => unreachable!(),
    }
}

fn coordinator_start_handler(args: &ArgMatches) {
    let database_url = args
        .get_one::<String>("db")
        .expect("Postgres database URL is required with --db flag.");

    let node = CoordinatorNode::new(database_url.as_ref());
    block_on(start_coordinator_server(Arc::new(node))).unwrap();
}

async fn start_coordinator_server(
    service: Arc<CoordinatorNode>,
) -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "[::]:2505".parse()?;
    tracing::info!("Starting coordinator server at port {}", addr.port());

    Server::builder()
        .add_service(CoordinatorNodeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

// Data action handlers.

pub fn data_handler(args: &ArgMatches) {
    match args.subcommand() {
        Some(("join", args)) => data_join_handler(args),
        _ => unreachable!(),
    }
}

fn data_join_handler(args: &ArgMatches) {
    let database_url = args
        .get_one::<String>("db")
        .expect("Please provide Postgres database URL with --db flag.");

    let coordinator_url = args
        .get_one::<String>("coordinator_url")
        .expect("Coordinator server URL is required to join the cluster.");

    let node = DataNode::new(database_url.as_ref(), coordinator_url.as_ref());
    block_on(join_data_server(Arc::new(node))).unwrap();
}

async fn join_data_server(
    service: Arc<DataNode>,
) -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "[::]:2510".parse()?;

    tracing::info!("Starting data node server at port {}", addr.port());
    tracing::info!("Joining coordinator: {}", service.coordinator_url());

    Server::builder()
        .add_service(DataNodeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
