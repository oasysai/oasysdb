mod db;
mod proto;
mod types;

#[cfg(test)]
mod tests;

use db::database::Database;
use proto::database_server::DatabaseServer;
use std::path::PathBuf;
use tonic::transport::Server;

const HOST: &str = "0.0.0.0";
const PORT: u16 = 2525;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{HOST}:{PORT}").parse()?;

    let path = PathBuf::from("/tmp/oasysdb");
    let database = Database::open(path)?;

    Server::builder()
        .add_service(DatabaseServer::new(database))
        .serve(addr)
        .await?;

    Ok(())
}
