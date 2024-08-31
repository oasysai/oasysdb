use super::*;
use oasysdb::nodes::{CoordinatorNode, DataNode};
use oasysdb::postgres::NodeParameters;
use oasysdb::protos::coordinator_node_server::CoordinatorNodeServer;
use oasysdb::protos::data_node_server::DataNodeServer;
use std::future::Future;
use tokio::runtime::Runtime;
use tonic::transport::Server;

fn block_on<F: Future>(future: F) -> F::Output {
    let rt = Runtime::new().expect("Failed to create a Tokio runtime");
    rt.block_on(future)
}

// Coordinator action handlers.

pub fn coordinator_handler(args: &ArgMatches) {
    match args.subcommand() {
        Some(("start", args)) => block_on(coordinator_start_handler(args)),
        _ => unreachable!(),
    }
}

async fn coordinator_start_handler(args: &ArgMatches) {
    let database_url = args.get_one::<Url>("db").unwrap().to_owned();
    let params = match args.get_one::<usize>("dim") {
        Some(dimension) => {
            let params = NodeParameters::new(*dimension);

            let params = match args.get_one::<Metric>("metric") {
                Some(metric) => params.with_metric(*metric),
                None => params,
            };

            let params = match args.get_one::<usize>("density") {
                Some(density) => params.with_density(*density),
                None => params,
            };

            Some(params)
        }
        None => None,
    };

    let node = CoordinatorNode::new(database_url, params).await;
    start_coordinator_server(Arc::new(node)).await.unwrap();
}

async fn start_coordinator_server(
    service: Arc<CoordinatorNode>,
) -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "[::]:2505".parse()?;
    tracing::info!("starting coordinator server at port {}", addr.port());

    Server::builder()
        .add_service(CoordinatorNodeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

// Data action handlers.

pub fn data_handler(args: &ArgMatches) {
    match args.subcommand() {
        Some(("join", args)) => block_on(data_join_handler(args)),
        _ => unreachable!(),
    }
}

async fn data_join_handler(args: &ArgMatches) {
    // Unwrap is safe because the argument is validated by clap.
    let name = args.get_one::<String>("name").unwrap().as_str();
    let database_url = args.get_one::<Url>("db").unwrap().to_owned();

    let coordinator_url = args
        .get_one::<SocketAddr>("coordinator_url")
        .expect("Coordinator address is required to join the cluster")
        .to_owned();

    let node = DataNode::new(name, coordinator_url, database_url).await;
    join_data_server(Arc::new(node)).await.unwrap();
}

async fn join_data_server(
    service: Arc<DataNode>,
) -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "[::]:2510".parse()?;
    tracing::info!("starting data node server at port {}", addr.port());

    Server::builder()
        .add_service(DataNodeServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
