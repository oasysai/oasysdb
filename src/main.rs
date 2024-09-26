mod types;

use clap::{arg, ArgMatches, Command};
use types::Metric;

fn main() {
    tracing_subscriber::fmt::init();

    let command = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interface to setup and manage OasysDB server")
        .arg_required_else_help(true)
        .subcommand(start())
        .subcommand(configure())
        .get_matches();

    match command.subcommand() {
        Some(("start", args)) => start_handler(args),
        Some(("configure", args)) => configure_handler(args),
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

pub fn start_handler(_args: &ArgMatches) {}

fn configure() -> Command {
    let arg_dimension = arg!(--dim <dimension> "Vector dimension")
        .required(true)
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false);

    // List optional arguments below.
    let arg_metric =
        arg!(--metric <metric> "Metric to calculate distance between vectors")
            .default_value(Metric::Euclidean.as_str())
            .value_parser(clap::value_parser!(Metric));

    Command::new("configure")
        .about("Configure the database parameters")
        .arg(arg_dimension)
        .arg(arg_metric)
}

pub fn configure_handler(_args: &ArgMatches) {}
