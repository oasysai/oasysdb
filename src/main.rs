use oasysdb::db::server::{Config, Server};

#[tokio::main]
async fn main() {
    // Define server parameters.
    let host = "127.0.0.1";
    let port = "3141";

    // Create the server configuration.
    let config = Config { dimension: 2 };

    // Create and start the server.
    let mut server = Server::new(host, port, config).await;
    server.serve().await;
}
