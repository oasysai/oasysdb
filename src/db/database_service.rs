use super::*;
use proto::database_server::Database as ProtoDatabase;

#[tonic::async_trait]
impl ProtoDatabase for Arc<Database> {
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
            Collection::validate_name(&field.name)?;

            // Use the MetadataType as a proxy to convert string to DataType.
            let metadata_type: MetadataType = field.datatype.into();
            let datatype: DataType = metadata_type.into();

            let new_field = Field::new(&field.name, datatype, true);
            fields.push(new_field);
        }

        self._add_fields(&request.collection_name, fields)?;
        Ok(Response::new(()))
    }

    async fn remove_fields(
        &self,
        request: Request<proto::RemoveFieldsRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        self._remove_fields(&request.collection_name, &request.field_names)?;
        Ok(Response::new(()))
    }

    async fn insert_records(
        &self,
        request: Request<proto::InsertRecordsRequest>,
    ) -> Result<Response<()>, Status> {
        let proto::InsertRecordsRequest {
            collection_name,
            field_names,
            records,
        } = request.into_inner();

        if field_names.is_empty() {
            return Err(Status::invalid_argument(
                "At least one field name must be specified.",
            ));
        }

        if !field_names.contains(&"vector".to_string()) {
            return Err(Status::invalid_argument(
                "The vector field must be specified.",
            ));
        }

        // Check if the records provided match the number of fields.
        // This is required since we try to simulate a batch insert like:
        // INSERT INTO collection_name (field1, field2)
        // VALUES
        // (x1, y1),
        // (x2, y2, z2) <- We should catch this error.
        if records
            .par_iter()
            .any(|record| record.data.len() != field_names.len())
        {
            let message = "The number of values must match the fields.";
            return Err(Status::invalid_argument(message));
        }

        let collection = self._get_collection(&collection_name)?;
        let schema = collection.state()?.schema;
        let fields = schema.fields;

        // Check if the fields specified in the request exist in the schema.
        if field_names.par_iter().any(|name| fields.find(name).is_none()) {
            return Err(Status::invalid_argument(
                "One or more fields specified do not exist in the schema.",
            ));
        }

        // Convert records from row format to column format.
        let mut columns = vec![vec![]; field_names.len()];
        for record in records {
            for (i, column) in columns.iter_mut().enumerate() {
                let value = record.data[i].value.clone();
                column.push(value);
            }
        }

        // Convert columns to Arrow arrays.
        let mut arrays = vec![];
        for i in 0..field_names.len() {
            let field = fields.find(&field_names[i]).unwrap().1;
            let column = columns[i].clone();
            let array = match field.data_type().clone().into() {
                MetadataType::Boolean => BooleanArray::from_values(column)?,
                MetadataType::Integer => Int32Array::from_values(column)?,
                MetadataType::Float => Float32Array::from_values(column)?,
                MetadataType::String => StringArray::from_values(column)?,
                MetadataType::Vector => ListArray::from_values(column)?,
            };

            arrays.push(array);
        }

        self._insert_records(&collection_name, &field_names, &arrays)?;
        Ok(Response::new(()))
    }
}
