use clap::{ArgMatches, Command};

mod actions;
mod commands;

pub fn run() {
    let command = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Interface to setup and manage OasysDB servers.")
        .arg_required_else_help(true)
        .subcommand(commands::coordinator())
        .subcommand(commands::data())
        .get_matches();

    match command.subcommand() {
        Some(("coordinator", args)) => actions::coordinator_handler(args),
        Some(("data", args)) => actions::data_handler(args),
        _ => unreachable!(),
    }
}