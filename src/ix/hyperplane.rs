use super::*;

pub struct Hyperplane<const N: usize> {
    pub coefficients: Vector<N>,
    pub constant: f32,
}

impl<const N: usize> Hyperplane<N> {
    pub fn point_is_above(&self, point: &Vector<N>) -> bool {
        self.coefficients.dot_product(point) + self.constant >= 0.0
    }

    pub fn build(
        keys: &Vec<&'static str>,
        vectors: &HashMap<&str, Vector<N>>,
    ) -> (Hyperplane<N>, Vec<&'static str>, Vec<&'static str>) {
        let mut rng = rand::thread_rng();
        let sample: Vec<_> = keys.choose_multiple(&mut rng, 2).collect();
        let (a, b) = (*sample[0], *sample[1]);

        let coefficients = vectors[a].subtract_from(&vectors[b]);
        let point_on_plane = vectors[a].average(&vectors[b]);
        let constant = -coefficients.dot_product(&point_on_plane);
        let hyperplane = Hyperplane::<N> { coefficients, constant };

        let mut left = vec![];
        let mut right = vec![];

        for key in keys.iter() {
            if hyperplane.point_is_above(&vectors[key]) {
                right.push(*key)
            } else {
                left.push(*key)
            };
        }

        (hyperplane, right, left)
    }
}
