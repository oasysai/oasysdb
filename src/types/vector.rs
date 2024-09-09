use super::*;

/// Vector data structure.
///
/// We use a boxed slice to store the vector data for a slight memory
/// efficiency boost. The length of the vector is not checked, so a length
/// validation should be performed before most operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector(Box<[f32]>);

impl Vector {
    /// Return the vector as a slice of floating-point numbers.
    pub fn as_slice(&self) -> &[f32] {
        self.0.as_ref()
    }

    /// Return as a vector of floating-point numbers.
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.to_vec()
    }

    /// Return the length of the vector.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return true if the vector is empty otherwise false.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// Vector conversion implementations.

impl From<Vec<f32>> for Vector {
    fn from(value: Vec<f32>) -> Self {
        Vector(value.into_boxed_slice())
    }
}
