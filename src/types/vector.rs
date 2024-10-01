use super::*;

/// Vector data structure.
///
/// We use a boxed slice to store the vector data for a slight memory
/// efficiency boost. The length of the vector is not checked, so a length
/// validation should be performed before most operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Vector(Box<[f32]>);

impl Vector {
    /// Return the vector as a slice of floating-point numbers.
    pub fn as_slice(&self) -> &[f32] {
        self.0.as_ref()
    }

    /// Return the length of the vector.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

// Vector conversion implementations.

impl From<Vec<f32>> for Vector {
    fn from(value: Vec<f32>) -> Self {
        Vector(value.into_boxed_slice())
    }
}

impl TryFrom<protos::Vector> for Vector {
    type Error = Status;
    fn try_from(value: protos::Vector) -> Result<Self, Self::Error> {
        Ok(Vector(value.data.into_boxed_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_vector() {
        let dim = 128;
        let vector = Vector::random(dim);
        assert_eq!(vector.len(), dim);
    }

    impl Vector {
        pub fn random(dimension: usize) -> Self {
            let vector = vec![0.0; dimension]
                .iter()
                .map(|_| rand::random::<f32>())
                .collect::<Vec<f32>>();

            Vector(vector.into_boxed_slice())
        }
    }
}
