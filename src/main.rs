// TODO: Remove this line when the code is ready
#![allow(dead_code)]

mod db;
mod proto;
mod types;

#[cfg(test)]
mod tests;

use db::database::Database;
use proto::database_server::DatabaseServer;
use tonic::transport::Server;

const HOST: &str = "0.0.0.0";
const PORT: u16 = 2525;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{HOST}:{PORT}").parse()?;
    let database = Database {};

    Server::builder()
        .add_service(DatabaseServer::new(database))
        .serve(addr)
        .await?;

    Ok(())
}
