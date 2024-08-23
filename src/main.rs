mod cli;

fn main() {
    tracing_subscriber::fmt::init();
    cli::run();
}
