use oasysdb::db::server::Server;

#[tokio::main]
async fn main() {
    let server = Server::new("127.0.0.1", "3141").await;
    server.serve().await;
}
