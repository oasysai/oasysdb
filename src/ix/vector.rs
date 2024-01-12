use super::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[derive(Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct VectorID(pub u32);

impl VectorID {
    /// True if this vector ID is valid.
    pub fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// The vector embedding where `N` is the vector dimension.
#[derive(Serialize, Deserialize, Clone)]
pub struct Vector<const N: usize>(#[serde(with = "BigArray")] pub [f32; N]);

impl<const N: usize> Vector<N> {
    /// Returns the Euclidean distance between two vectors.
    pub fn distance(&self, other: &Self) -> f32 {
        let iter = self.0.iter().zip(other.0.iter());
        iter.map(|(a, b)| (a - b).powi(2)).sum::<f32>().sqrt()
    }
}

impl<const N: usize> Index<&VectorID> for [Vector<N>] {
    type Output = Vector<N>;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self[index.0 as usize]
    }
}
