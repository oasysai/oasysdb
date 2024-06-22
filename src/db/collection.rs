use super::*;
use arrow::array::RecordBatch;
use arrow::datatypes::{Field, Schema};
use std::sync::{Arc, RwLock};

pub type ArcLock<T> = Arc<RwLock<T>>;

pub struct Collection {
    schema: ArcLock<Schema>,
    data: ArcLock<Vec<RecordBatch>>,
}

impl Collection {
    pub fn new() -> Self {
        let schema = Arc::new(RwLock::new(Schema::empty()));
        let data = Arc::new(RwLock::new(vec![]));
        Self { schema, data }
    }
}
