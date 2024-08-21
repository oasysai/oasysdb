use super::*;

/// ID for a vector record in the database.
///
/// We use UUID version 4 for the record ID to ensure uniqueness accross the
/// distributed data store. The drawback is that UUIDs occupy 2 to 4 times more
/// space than integers.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RecordID(Uuid);

impl RecordID {
    /// Create a new record ID using UUID version 4.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
