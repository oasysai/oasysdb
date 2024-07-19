use crate::types::distance::DistanceMetric;
use crate::types::record::Vector;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::rc::Rc;

pub type Vectors<'v> = Rc<[&'v Vector]>;

#[derive(Debug, Clone, Copy, Default, Hash)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ClusterID(pub u16);

#[derive(Debug)]
pub struct KMeans {
    num_centroids: usize,
    num_iterations: usize,
    metric: DistanceMetric,
    assignment: Vec<ClusterID>, // Cluster assignment for each vector.
    centroids: Vec<Vector>,     // Centroids of each cluster.
}

impl KMeans {
    /// Creates a new KMeans model.
    pub fn new(
        num_centroids: usize,
        num_iterations: usize,
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
    pub fn fit(&mut self, vectors: Vectors) {
        // Cloning the vectors is acceptable because with Rc, we are
        // only cloning the references, not the actual data.
        self.centroids = self.initialize_centroids(vectors.clone());

        let mut repeat_centroids = 0;
        for _ in 0..self.num_iterations {
            if repeat_centroids > 5 {
                break;
            }

            self.assignment = self.assign_clusters(vectors.clone());
            let centroids = self.update_centroids(vectors.clone());

            match self.centroids == centroids {
                true => repeat_centroids += 1,
                false => {
                    self.centroids = centroids;
                    repeat_centroids = 0;
                }
            }
        }
    }

    fn initialize_centroids(&self, vectors: Vectors) -> Vec<Vector> {
        let mut rng = rand::thread_rng();
        vectors
            .choose_multiple(&mut rng, self.num_centroids)
            .cloned()
            .map(|vector| vector.to_owned())
            .collect()
    }

    fn assign_clusters(&self, vectors: Vectors) -> Vec<ClusterID> {
        vectors
            .into_par_iter()
            .map(|vector| self.find_nearest_centroid(vector))
            .collect()
    }

    fn update_centroids(&self, vectors: Vectors) -> Vec<Vector> {
        let k = self.num_centroids;
        let mut counts = vec![0; k];

        let mut centroids = {
            let dimension = vectors[0].len();
            let zeros = vec![0.0; dimension];
            vec![zeros; k]
        };

        for (i, vector) in vectors.iter().enumerate() {
            let cluster_id = self.assignment[i].0 as usize;
            counts[cluster_id] += 1;

            centroids[cluster_id]
                .par_iter_mut()
                .zip(vector.to_vec().par_iter())
                .for_each(|(sum, v)| {
                    *sum += v;
                });
        }

        for i in 0..k {
            if counts[i] == 0 {
                let mut rng = rand::thread_rng();
                centroids[i] = vectors.choose(&mut rng).unwrap().to_vec();
                continue;
            }

            centroids[i].par_iter_mut().for_each(|sum| {
                *sum /= counts[i] as f32;
            });
        }

        centroids.into_iter().map(|v| v.into()).collect()
    }

    /// Finds the nearest centroid to a given vector.
    /// - `vector`: Vector to compare with the centroids.
    pub fn find_nearest_centroid(&self, vector: &Vector) -> ClusterID {
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

        let vectors: Vectors = {
            let vectors_ref: Vec<&Vector> = vectors.iter().collect();
            Rc::from(vectors_ref.as_slice())
        };

        let mut kmeans = KMeans::new(5, 20, DistanceMetric::Euclidean);
        kmeans.fit(vectors.clone());

        let mut correct_count = 0;
        for (i, clusted_id) in kmeans.assignments().iter().enumerate() {
            let vector = vectors[i];
            let nearest_centroid = kmeans.find_nearest_centroid(vector);
            if clusted_id == &nearest_centroid {
                correct_count += 1;
            }
        }

        let accuracy = correct_count as f32 / vectors.len() as f32;
        assert!(accuracy > 0.95);
    }
}
