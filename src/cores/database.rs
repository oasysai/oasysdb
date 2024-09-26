use super::*;
use crate::protos::database_server::Database as DatabaseService;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct Database {}

impl Database {
    pub fn configure() {}

    pub fn open() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl DatabaseService for Arc<Database> {
    async fn heartbeat(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}
