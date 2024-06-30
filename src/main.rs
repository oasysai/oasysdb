#![allow(clippy::enum_variant_names)]

mod db;
mod proto;
mod types;

#[cfg(test)]
mod tests;

use db::*;
use proto::database_server::DatabaseServer;
use std::path::PathBuf;
use std::sync::Arc;
use tonic::transport::Server;

const HOST: &str = "0.0.0.0";
const PORT: u16 = 2525;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{HOST}:{PORT}").parse()?;

    let path = PathBuf::from("odb_data");
    let database = Arc::new(Database::open(path)?);

    Server::builder()
        .add_service(DatabaseServer::new(database))
        .serve(addr)
        .await?;

    Ok(())
}
