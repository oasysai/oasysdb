use super::*;
use clap::arg;

// Coordinator subcommands section.

pub fn coordinator() -> Command {
    Command::new("coordinator")
        .about("Interface to manage the coordinator server")
        .subcommand_required(true)
        .subcommand(coordinator_start())
        .subcommand(coordinator_config())
}

fn coordinator_start() -> Command {
    Command::new("start")
        .alias("run")
        .about("Start server as the coordinator node")
}

fn coordinator_config() -> Command {
    let arg_metric =
        arg!(--metric <metric> "Metric to calculate distance between vectors")
            .default_value(Metric::Euclidean.as_str())
            .value_parser(clap::value_parser!(Metric));

    let arg_dimension = arg!(--dim <dimension> "Vector dimension")
        .required(true)
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false);

    let arg_density = arg!(--density <density> "Number of records per cluster")
        .default_value("128")
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false);

    Command::new("config")
        .about("Configure the coordinator node parameters")
        .arg(arg_metric)
        .arg(arg_dimension)
        .arg(arg_density)
}

// Data subcommands section.

pub fn data() -> Command {
    Command::new("data")
        .about("Interface to manage the data server")
        .subcommand_required(true)
        .subcommand(data_join())
}

fn data_join() -> Command {
    let arg_name = arg!(<name> "Name of the data node").required(true);

    let arg_coordinator_addr =
        arg!(<coordinator_addr> "Coordinator server address")
            .required(true)
            .value_parser(clap::value_parser!(SocketAddr));

    let arg_port = arg!(--port <port> "Port to listen on")
        .default_value("2510")
        .value_parser(clap::value_parser!(u16))
        .allow_negative_numbers(false);

    Command::new("join")
        .about("Start and join server as a data node in the cluster")
        .arg(arg_name)
        .arg(arg_coordinator_addr)
        .arg(arg_port)
}
