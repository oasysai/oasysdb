mod cores;
mod protos;
mod types;
mod utils;

use clap::{arg, ArgMatches, Command};
use cores::{Database, Parameters};
use dotenv::dotenv;
use protos::database_server::DatabaseServer;
use std::sync::Arc;
use tonic::transport::Server;
use types::Metric;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let command = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interface to setup and manage OasysDB server")
        .arg_required_else_help(true)
        .subcommand(start())
        .subcommand(configure())
        .get_matches();

    match command.subcommand() {
        Some(("start", args)) => start_handler(args).await,
        Some(("configure", args)) => configure_handler(args).await,
        _ => unreachable!(),
    }
}

fn start() -> Command {
    let arg_port = arg!(--port <port> "Port to listen on")
        .default_value("2505")
        .value_parser(clap::value_parser!(u16))
        .allow_negative_numbers(false);

    Command::new("start")
        .alias("run")
        .about("Start the database server")
        .arg(arg_port)
}

async fn start_handler(args: &ArgMatches) {
    // Unwrap is safe because Clap validates the arguments.
    let port = args.get_one::<u16>("port").unwrap();
    let addr = format!("[::]:{port}").parse().unwrap();

    let database = Database::open().expect("Failed to open the database");
    let service = DatabaseServer::new(Arc::new(database));

    tracing::info!("The database server is ready on port {port}");

    Server::builder()
        .add_service(service)
        .serve(addr)
        .await
        .expect("Failed to start the database");
}

fn configure() -> Command {
    let arg_dimension = arg!(--dim <dimension> "Vector dimension")
        .required(true)
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false);

    // List optional arguments below.
    let arg_metric = arg!(--metric <metric> "Metric to calculate distance")
        .default_value(Metric::Euclidean.as_str())
        .value_parser(clap::value_parser!(Metric));

    let arg_density = arg!(--density <density> "Density of the cluster")
        .default_value("256")
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false);

    Command::new("configure")
        .about("Configure the initial database parameters")
        .arg(arg_dimension)
        .arg(arg_metric)
        .arg(arg_density)
}

async fn configure_handler(args: &ArgMatches) {
    let dim = *args.get_one::<usize>("dim").unwrap();
    let metric = *args.get_one::<Metric>("metric").unwrap();
    let density = *args.get_one::<usize>("density").unwrap();

    let params = Parameters { dimension: dim, metric, density };
    Database::configure(&params);
}
