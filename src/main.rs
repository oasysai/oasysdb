use oasysdb::db::server::Server;

#[tokio::main]
async fn main() {
    // Define server parameters.
    let host = "127.0.0.1";
    let port = "3141";
    let dimension = 3;

    // Create and start the server.
    let mut server = Server::new(host, port, dimension).await;
    server.serve().await;
}
