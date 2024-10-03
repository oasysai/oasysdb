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

        let vector = Vector::random(128);
        let record = Record { vector, metadata: HashMap::new() };
        let id = RecordID::new();
        storage.insert(&id, &record).unwrap();

        assert_eq!(storage.count, 1);
        assert_eq!(storage.count, storage.records.len());
    }
}
