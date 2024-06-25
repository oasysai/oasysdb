use super::*;
use arrow::array::RecordBatch;
use arrow::datatypes::Fields;
use arrow_schema::Schema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionState {
    pub schema: Schema,
    pub count: usize,
}

impl Default for CollectionState {
    fn default() -> Self {
        Self { schema: Schema::empty(), count: 0 }
    }
}

pub struct Collection {
    data: Lock<Vec<RecordBatch>>,
    state: Lock<CollectionState>,
}

impl Collection {
    pub fn new() -> Result<Self, Error> {
        let data = Lock::new(vec![]);
        let state = Lock::new(CollectionState::default());
        let collection = Self { data, state };
        Ok(collection)
    }

    pub fn add_fields(&self, fields: impl Into<Fields>) -> Result<(), Error> {
        let mut state = self.state.write()?;

        // Create a new schema with the new field.
        let schema = &state.schema;
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

        // Update the state and data.
        state.schema = new_schema;
        *state = state.clone();
        *data = migrated_data;

        Ok(())
    }

    pub fn state(&self) -> Result<CollectionState, Error> {
        Ok(self.state.read()?.clone())
    }
}
