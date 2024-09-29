use super::*;

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    centroids: Vec<Vector>,
    clusters: Vec<Vec<RecordID>>,
}

impl Index {
    pub fn new() -> Self {
        Index { centroids: vec![], clusters: vec![] }
    }
}
