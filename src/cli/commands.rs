use super::*;
use clap::{arg, Arg};

// Coordinator subcommands section.

pub fn coordinator() -> Command {
    Command::new("coordinator")
        .about("Interface to manage the coordinator server")
        .subcommand_required(true)
        .subcommand(coordinator_start())
}

fn coordinator_start() -> Command {
    Command::new("start")
        .about("Start server as the coordinator node")
        .arg(shared_arg_database_url())
        .arg(coordinator_arg_metric())
        .arg(coordinator_arg_dimension())
        .arg(coordinator_arg_density())
}

fn coordinator_arg_dimension() -> Arg {
    arg!(--dim <dimension> "Vector dimension")
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false)
}

fn coordinator_arg_metric() -> Arg {
    arg!(--metric <metric> "Metric to calculate distance between vectors")
        .default_value(Metric::Euclidean.as_str())
        .value_parser(clap::value_parser!(Metric))
}

fn coordinator_arg_density() -> Arg {
    arg!(--density <density> "Number of records per cluster")
        .default_value("128")
        .value_parser(clap::value_parser!(usize))
        .allow_negative_numbers(false)
}

// Data subcommands section.

pub fn data() -> Command {
    Command::new("data")
        .about("Interface to manage the data server")
        .subcommand_required(true)
        .subcommand(data_join())
}

fn data_join() -> Command {
    Command::new("join")
        .about("Start and join server as a data node in the cluster")
        .arg(data_arg_name())
        .arg(data_arg_coordinator_addr())
        .arg(shared_arg_database_url())
}

fn data_arg_name() -> Arg {
    arg!(<name> "Name of the data node").required(true)
}

fn data_arg_coordinator_addr() -> Arg {
    arg!(<coordinator_addr> "Coordinator server address")
        .required(true)
        .value_parser(clap::value_parser!(SocketAddr))
}

// Shared arguments section.

fn shared_arg_database_url() -> Arg {
    arg!(--db <database_url> "PostgreSQL database URL")
        .required(true)
        .value_parser(clap::value_parser!(Url))
}
