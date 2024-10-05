use super::*;

/// Record storage interface.
///
/// This interface wraps around Hashbrown's HashMap implementation to store
/// the records. In the future, if needed, we can modify the storage
/// implementation without changing the rest of the code.
#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    count: usize,
    records: HashMap<RecordID, Record>,
}

impl Storage {
    /// Create a new empty storage instance.
    pub fn new() -> Self {
        Storage { count: 0, records: HashMap::new() }
    }

    /// Insert a new record into the record storage.
    pub fn insert(
        &mut self,
        id: &RecordID,
        record: &Record,
    ) -> Result<(), Status> {
        self.records.insert(*id, record.to_owned());
        self.count += 1;
        Ok(())
    }

    /// Delete a record from the storage given its ID.
    pub fn delete(&mut self, id: &RecordID) -> Result<(), Status> {
        self.records.remove(id);
        self.count -= 1;
        Ok(())
    }

    /// Update a record metadata given its ID.
    ///
    /// Vector data should be immutable as it is tightly coupled with the
    /// semantic meaning of the record. If the vector data changes, users
    /// should create a new record instead.
    pub fn update(
        &mut self,
        id: &RecordID,
        metadata: &HashMap<String, Value>,
    ) -> Result<(), Status> {
        let record = match self.records.get_mut(id) {
            Some(record) => record,
            None => {
                let message = "The specified record is not found";
                return Err(Status::not_found(message));
            }
        };

        record.metadata = metadata.to_owned();
        Ok(())
    }

    /// Return a reference to the records in the storage.
    pub fn records(&self) -> &HashMap<RecordID, Record> {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut storage = Storage::new();

        let record = Record::random(128);
        let id = RecordID::new();
        storage.insert(&id, &record).unwrap();

        assert_eq!(storage.count, 1);
        assert_eq!(storage.count, storage.records.len());
    }

    #[test]
    fn test_delete() {
        let mut storage = Storage::new();

        let record = Record::random(128);
        let id = RecordID::new();
        storage.insert(&id, &record).unwrap();

        storage.delete(&id).unwrap();
        assert_eq!(storage.count, 0);
        assert_eq!(storage.count, storage.records.len());
    }

    #[test]
    fn test_update() {
        let mut storage = Storage::new();

        let record = Record::random(128);
        let id = RecordID::new();
        storage.insert(&id, &record).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), Value::random());
        storage.update(&id, &metadata).unwrap();

        let updated_record = storage.records.get(&id).unwrap();
        assert_eq!(updated_record.metadata, metadata);
    }
}
