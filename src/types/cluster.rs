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
