use super::*;

/// A subspace of the vector space often used to divide the vectors.
pub struct Hyperplane<const N: usize> {
    pub coefficients: Vector<N>,
    pub constant: f32,
}

impl<const N: usize> Hyperplane<N> {
    /// Calculates if the point is above the hyperplane.
    pub fn point_is_above(&self, point: &Vector<N>) -> bool {
        self.coefficients.dot_product(point) + self.constant >= 0.0
    }

    /// Creates a hyperplane that divides the vectors into two groups.
    /// * `keys` - The keys of vectors to divide.
    /// * `vectors` - Mapping of keys to vectors.
    pub fn build(
        keys: &Vec<&'static str>,
        vectors: &HashMap<&str, Vector<N>>,
    ) -> (Hyperplane<N>, Vec<&'static str>, Vec<&'static str>) {
        // The size of the vectors and keys can differ.
        // Only the vectors which key is in keys are used.

        // Pick two random keys.
        let mut rng = rand::thread_rng();
        let sample: Vec<&&str> = keys.choose_multiple(&mut rng, 2).collect();
        let (a, b) = (*sample[0], *sample[1]);
        let (vec_a, vec_b) = (vectors[a], vectors[b]);

        // Calculate the coefficients and constant of the hyperplane.
        let coefficients = vec_a.subtract_from(&vec_b);
        let middle_point = vec_a.average(&vec_b);
        let constant = -coefficients.dot_product(&middle_point);
        let hyperplane = Hyperplane::<N> { coefficients, constant };

        let mut above = vec![];
        let mut below = vec![];

        // Collect the keys which vector is above or below the hyperplane.
        for key in keys.iter() {
            if hyperplane.point_is_above(&vectors[key]) {
                above.push(*key)
            } else {
                below.push(*key)
            };
        }

        (hyperplane, above, below)
    }
}
