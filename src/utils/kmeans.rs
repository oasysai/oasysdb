use crate::types::distance::DistanceMetric;
use crate::types::record::Vector;
use rand::Rng;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, Default, Hash)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ClusterID(pub u16);

#[derive(Debug)]
pub struct KMeans {
    num_centroids: usize,
    num_iterations: u8,
    metric: DistanceMetric,
    assignment: Vec<ClusterID>, // Cluster assignment for each vector.
    centroids: Vec<Vector>,     // Centroids of each cluster.
}

impl KMeans {
    /// Creates a new KMeans model.
    pub fn new(
        num_centroids: usize,
        num_iterations: u8,
        metric: DistanceMetric,
    ) -> Self {
        Self {
            num_centroids,
            num_iterations,
            metric,
            assignment: vec![],
            centroids: vec![],
        }
    }

    /// Fits the KMeans model to the given vectors.
    pub fn fit(&mut self, vectors: &[Vector]) {
        self.centroids = self.initialize_centroids(vectors);

        for _ in 0..self.num_iterations {
            self.assignment = self.assign_clusters(vectors);
            self.centroids = self.update_centroids(vectors);
        }
    }

    fn initialize_centroids(&self, vectors: &[Vector]) -> Vec<Vector> {
        let mut rng = rand::thread_rng();
        let mut centroids = Vec::with_capacity(self.num_centroids);
        for _ in 0..self.num_centroids {
            let index = rng.gen_range(0..vectors.len());
            centroids.push(vectors[index].to_owned());
        }

        centroids
    }

    fn assign_clusters(&self, vectors: &[Vector]) -> Vec<ClusterID> {
        let assign = |vector| self.find_closest_centroid(vector);
        vectors.par_iter().map(assign).collect()
    }

    fn update_centroids(&self, vectors: &[Vector]) -> Vec<Vector> {
        let k = self.num_centroids;
        let mut counts = vec![0; k];

        let mut sums = {
            let dimension = vectors[0].len();
            let zeros = vec![0.0; dimension];
            vec![zeros; k]
        };

        for (i, vector) in vectors.iter().enumerate() {
            let cluster_id = self.assignment[i].0 as usize;
            counts[cluster_id] += 1;

            sums[cluster_id]
                .par_iter_mut()
                .zip(vector.to_vec().par_iter())
                .for_each(|(sum, v)| {
                    *sum += v;
                });
        }

        for i in 0..self.num_centroids {
            sums[i].par_iter_mut().for_each(|sum| {
                *sum /= counts[i] as f32;
            });
        }

        sums.into_iter().map(|v| v.into()).collect()
    }

    /// Finds the closest centroid to a given vector.
    /// - `vector`: Vector to compare with the centroids.
    pub fn find_closest_centroid(&self, vector: &Vector) -> ClusterID {
        self.centroids
            .par_iter()
            .map(|centroid| self.metric.distance(vector, centroid))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| ClusterID(i as u16))
            .unwrap_or_default()
    }

    /// Returns the cluster assignment for each vector.
    pub fn assignments(&self) -> &[ClusterID] {
        &self.assignment
    }

    /// Returns the centroids of each cluster.
    pub fn centroids(&self) -> &[Vector] {
        &self.centroids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmeans_fit() {
        let mut vectors = vec![];
        for i in 0..100 {
            let vector = Vector::from(vec![i as f32; 2]);
            vectors.push(vector);
        }

        let mut kmeans = KMeans::new(5, 20, DistanceMetric::Euclidean);
        kmeans.fit(&vectors);

        let mut correct_count = 0;
        for (i, clusted_id) in kmeans.assignments().iter().enumerate() {
            let vector = &vectors[i];
            let closest_centroid = kmeans.find_closest_centroid(vector);
            if clusted_id == &closest_centroid {
                correct_count += 1;
            }
        }

        let accuracy = correct_count as f32 / vectors.len() as f32;
        assert!(accuracy > 0.95);
    }
}
