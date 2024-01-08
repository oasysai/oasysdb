type HashKey<const N: usize> = [u32; N];

/// The vector embedding of the node where `N` is the vector dimension.
pub type Vector<const N: usize> = [f32; N];

/// Additional utility methods for vectors.
pub trait VectorOps<const N: usize> {
    /// Calculates the dot product of two vectors.
    fn dot_product(&self, vector: &Vector<N>) -> f32;
    /// Calculates the distance between vectors using the Euclidean metric.
    fn euclidean_distance(&self, vector: &Vector<N>) -> f32;
    /// Subtracts the vector from another vector.
    fn subtract_from(&self, vector: &Vector<N>) -> Vector<N>;
    /// Calculates the middle plane between two vectors.
    fn average(&self, vector: &Vector<N>) -> Vector<N>;
    /// Converts the vector to a vector of bits.
    fn to_hashkey(&self) -> HashKey<N>;
}

impl<const N: usize> VectorOps<N> for Vector<N> {
    fn dot_product(&self, vector: &Vector<N>) -> f32 {
        let iter = self.iter().zip(vector);
        iter.map(|(a, b)| a * b).sum()
    }

    fn euclidean_distance(&self, vector: &Vector<N>) -> f32 {
        let iter = self.iter().zip(vector);
        let sum: f32 = iter.map(|(a, b)| (a - b).powi(2)).sum();
        sum.sqrt()
    }

    fn subtract_from(&self, vector: &Vector<N>) -> Vector<N> {
        let results = self.iter().zip(vector).map(|(a, b)| b - a);
        results.collect::<Vec<f32>>().try_into().unwrap()
    }

    fn average(&self, vector: &Vector<N>) -> Vector<N> {
        let results = self.iter().zip(vector).map(|(a, b)| (a + b) / 2.0);
        results.collect::<Vec<f32>>().try_into().unwrap()
    }

    fn to_hashkey(&self) -> HashKey<N> {
        let iter = self.iter().map(|a| a.to_bits());
        iter.collect::<Vec<u32>>().try_into().unwrap()
    }
}
