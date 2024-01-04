type HashKey<const N: usize> = [u32; N];

pub type Vector<const N: usize> = [f32; N];

pub trait VectorOps<const N: usize> {
    fn dot_product(&self, vector: &Vector<N>) -> f32;
    fn euclidean_distance(&self, vector: &Vector<N>) -> f32;
    fn subtract_from(&self, vector: &Vector<N>) -> Vector<N>;
    fn average(&self, vector: &Vector<N>) -> Vector<N>;
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
        results.collect::<Vec<_>>().try_into().unwrap()
    }

    fn average(&self, vector: &Vector<N>) -> Vector<N> {
        let results = self.iter().zip(vector).map(|(a, b)| (a + b) / 2.0);
        results.collect::<Vec<_>>().try_into().unwrap()
    }

    fn to_hashkey(&self) -> HashKey<N> {
        let iter = self.iter().map(|a| a.to_bits());
        iter.collect::<Vec<_>>().try_into().unwrap()
    }
}
