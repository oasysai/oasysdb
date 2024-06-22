use super::*;
use crate::proto::database_server::Database as ProtoDatabase;
use crate::proto::CreateCollectionRequest;

pub struct Database {}

#[tonic::async_trait]
impl ProtoDatabase for Database {
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        unimplemented!();
    }
}
