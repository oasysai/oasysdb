use super::*;

/// The ID of a vector record.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[derive(Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct VectorID(pub u32);

impl VectorID {
    /// True if this vector ID is valid.
    pub fn is_valid(&self) -> bool {
        self.0 != u32::MAX
    }
}

/// The vector embedding where `N` is the vector dimension.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
pub struct Vector(pub Vec<f32>);

impl Vector {
    /// Returns the Euclidean distance between two vectors.
    pub fn distance(&self, other: &Self) -> f32 {
        assert_eq!(self.0.len(), other.0.len());
        let iter = self.0.iter().zip(other.0.iter());
        iter.map(|(a, b)| (a - b).powi(2)).sum::<f32>().sqrt()
    }
}

impl Index<&VectorID> for [Vector] {
    type Output = Vector;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self[index.0 as usize]
    }
}
