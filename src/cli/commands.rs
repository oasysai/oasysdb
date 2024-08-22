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
        .arg(arg!(<coordinator_url> "Coordinator server URL"))
        .arg(shared_arg_database_url())
}

// Shared arguments section.

fn shared_arg_database_url() -> Arg {
    arg!(--db <database_url> "PostgreSQL database URL")
}
