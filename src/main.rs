use oasysdb::db::server::Server;

#[tokio::main]
async fn main() {
    // Define server parameters.
    let host = "127.0.0.1";
    let port = "3141";

    // Create and start the server.
    let mut server = Server::new(host, port).await;
    server.serve().await;
}
