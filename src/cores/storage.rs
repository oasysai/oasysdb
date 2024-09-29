use super::*;

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    count: usize,
    records: HashMap<RecordID, Record>,
}

impl Storage {
    pub fn new() -> Self {
        Storage { count: 0, records: HashMap::new() }
    }
}
