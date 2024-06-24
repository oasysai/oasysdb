use super::database::Database;
use super::*;
use crate::proto::database_server::Database as ProtoDatabase;
use crate::proto::*;

#[tonic::async_trait]
impl ProtoDatabase for Database {
    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        self._create_collection(&request.name)?;
        Ok(Response::new(()))
    }

    async fn delete_collection(
        &self,
        request: Request<DeleteCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        self._delete_collection(&request.name)?;
        Ok(Response::new(()))
    }
}
