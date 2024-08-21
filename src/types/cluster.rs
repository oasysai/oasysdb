use super::*;

/// Unique ID for a data node cluster.
///
/// The underlying data structure uses UUID version 4, which is a randomly
/// generated 128-bit number. This ensures that the cluster ID is unique across
/// the distributed IVF indices.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ClusterID(Uuid);

impl ClusterID {
    /// Create a new random cluster ID using UUID version 4.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ClusterID {
    fn default() -> Self {
        Self::new()
    }
}

/// Cluster data structure in the data node.
///
/// Fields:
/// - centroid: Vector representing the cluster.
/// - members: List of record IDs in the cluster.
#[derive(Debug)]
pub struct Cluster {
    centroid: Vector,
    members: Vec<RecordID>,
}

impl Cluster {
    /// Return the centroid vector of the cluster.
    pub fn centroid(&self) -> &Vector {
        &self.centroid
    }

    /// Return a slice of record IDs that belong in the cluster.
    pub fn members(&self) -> &[RecordID] {
        &self.members
    }
}
