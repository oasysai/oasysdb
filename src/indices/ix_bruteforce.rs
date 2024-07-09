use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexBruteForce {
    config: SourceConfig,
    data: HashMap<RecordID, Record>,
    hidden: Vec<RecordID>,
}
