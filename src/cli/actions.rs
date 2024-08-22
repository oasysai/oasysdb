use super::*;

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
}
