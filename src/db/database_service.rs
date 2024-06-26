use super::*;
use crate::proto;
use crate::proto::database_server::Database as ProtoDatabase;

#[tonic::async_trait]
impl ProtoDatabase for Database {
    async fn create_collection(
        &self,
        request: Request<proto::CreateCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        self._create_collection(&request.name)?;
        Ok(Response::new(()))
    }

    async fn delete_collection(
        &self,
        request: Request<proto::DeleteCollectionRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        self._delete_collection(&request.name)?;
        Ok(Response::new(()))
    }

    async fn add_fields(
        &self,
        request: Request<proto::AddFieldsRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        // Construct Arrow fields from the request fields.
        let mut fields = vec![];
        for field in request.fields {
            // Use the MetadataType as a proxy to convert string to DataType.
            let metadata_type: MetadataType = field.datatype.into();
            let datatype: DataType = metadata_type.into();
            let new_field = Field::new(&field.name, datatype, true);
            fields.push(new_field);
        }

        self._add_fields(&request.collection_name, fields)?;
        Ok(Response::new(()))
    }
}
