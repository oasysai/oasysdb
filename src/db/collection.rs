use super::*;
use arrow::array::RecordBatch;
use arrow::datatypes::{Fields, Schema};

pub struct Collection {
    schema: Lock<Schema>,
    data: Lock<Vec<RecordBatch>>,
    count: Lock<usize>,
}

impl Collection {
    pub fn new() -> Self {
        let schema = Lock::new(Schema::empty());
        let data = Lock::new(vec![]);
        let count = Lock::new(0);
        Self { schema, data, count }
    }

    pub fn add_fields(&self, fields: impl Into<Fields>) -> Result<(), Error> {
        // Create a new schema with the new field.
        let mut schema = self.schema.write()?;
        let schemas = vec![schema.clone(), Schema::new(fields)];
        let new_schema = Schema::try_merge(schemas)?;

        // Migrate the data to the new schema.
        let migrate_data = |batch: &RecordBatch| {
            let schema = Arc::new(new_schema.clone());

            // We can unwrap here because the new schema is guaranted
            // to be a superset of the old schema.
            batch.clone().with_schema(schema).unwrap()
        };

        let mut data = self.data.write()?;
        let migrated_data = data.par_iter().map(migrate_data).collect();

        // Update the schema and data.
        *schema = new_schema;
        *data = migrated_data;

        Ok(())
    }

    pub fn count(&self) -> usize {
        *self.count.read().unwrap()
    }

    pub fn schema(&self) -> Result<Schema, Error> {
        let schema = self.schema.read()?;
        Ok(schema.clone())
    }
}
