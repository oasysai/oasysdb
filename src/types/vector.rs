use super::*;

/// Vector data structure.
///
/// We use a boxed slice to store the vector data for a slight memory
/// efficiency boost. The length of the vector is not checked, so a length
/// validation should be performed before most operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Vector(Box<[f32]>);
