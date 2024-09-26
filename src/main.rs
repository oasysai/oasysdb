mod cores;
mod protos;
mod types;

use clap::{arg, ArgMatches, Command};
use cores::Database;
use protos::database_server::DatabaseServer;
use std::sync::Arc;
use tonic::transport::Server;
use types::Metric;

#[tokio::main]
async fn main() {
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
    let arg_port = arg!(-p --port <port> "Port to listen on")
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

    let database = Database::open();
    let service = DatabaseServer::new(Arc::new(database));

    Server::builder()
        .add_service(service)
        .serve(addr)
        .await
        .expect("Failed to start the database");
}

fn configure() -> Command {
    let arg_dimension = arg!(-d --dim <dimension> "Vector dimension")
        .required(true)
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false);

    // List optional arguments below.
    let arg_metric = arg!(-m --metric <metric> "Metric to calculate distance")
        .default_value(Metric::Euclidean.as_str())
        .value_parser(clap::value_parser!(Metric));

    Command::new("configure")
        .about("Configure the initial database parameters")
        .arg(arg_dimension)
        .arg(arg_metric)
}

async fn configure_handler(args: &ArgMatches) {}
