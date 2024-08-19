use super::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::min;
use std::rc::Rc;

type ClusterIndex = usize;

/// A list of vectors.
///
/// We use a reference-counted slice to store the vectors. This allows us to
/// share the vectors around without having to actually clone the vectors.
type Vectors<'v> = Rc<[&'v Vector]>;

/// K-means clustering algorithm.
///
/// The K-means algorithm is a clustering algorithm that partitions a dataset
/// into K clusters by iteratively assigning data points to the nearest cluster
/// centroids and recalculating these centroids until they are stable.
#[derive(Debug)]
pub struct KMeans {
    assignments: Vec<ClusterIndex>,
    centroids: Vec<Vector>,

    // Algorithm parameters.
    metric: Metric,
    n_clusters: usize,
    max_iter: usize,
}

impl KMeans {
    /// Initialize the K-means algorithm with default parameters.
    ///
    /// Default parameters:
    /// - metric: Euclidean
    /// - max_iter: 300
    pub fn new(n_clusters: usize) -> Self {
        Self {
            n_clusters,
            metric: Metric::Euclidean,
            max_iter: 300,
            assignments: Vec::new(),
            centroids: Vec::with_capacity(n_clusters),
        }
    }

    /// Configure the metric used for distance calculations.
    pub fn with_metric(mut self, metric: Metric) -> Self {
        self.metric = metric;
        self
    }

    /// Configure the maximum number of iterations to run the algorithm.
    pub fn with_max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    /// Train the K-means algorithm with the given vectors.
    pub fn fit(&mut self, vectors: Vectors) -> Result<()> {
        if self.n_clusters > vectors.len() {
            let code = ErrorCode::InvalidParameter;
            let message = "Dataset is smaller than cluster configuration.";
            let error = Error::new(code, message)
                .with_action("Increase dataset size or reduce cluster count.");

            return Err(error);
        }

        self.centroids = self.initialize_centroids(vectors.clone());
        self.assignments = vec![0; vectors.len()];

        let mut no_improvement_count = 0;
        for _ in 0..self.max_iter {
            if no_improvement_count > 5 {
                break;
            }

            let assignments = self.assign_clusters(vectors.clone());

            // Check at most 1000 assignments for convergence.
            // This prevents checking the entire dataset for large datasets.
            let end = min(1000, assignments.len());
            match assignments[0..end] == self.assignments[0..end] {
                true => no_improvement_count += 1,
                false => no_improvement_count = 0,
            }

            self.assignments = assignments;
            self.centroids = self.update_centroids(vectors.clone());
        }

        Ok(())
    }

    fn initialize_centroids(&self, vectors: Vectors) -> Vec<Vector> {
        let mut rng = rand::thread_rng();
        let mut centroids = Vec::with_capacity(self.n_clusters);

        // Pick the first centroid randomly.
        let first_centroid = vectors.choose(&mut rng).cloned().unwrap();
        centroids.push(first_centroid.to_owned());

        for _ in 1..self.n_clusters {
            let nearest_centroid_distance = |vector: &&Vector| {
                centroids
                    .iter()
                    .map(|centroid| self.metric.distance(vector, centroid))
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap()
            };

            let distances = vectors
                .par_iter()
                .map(nearest_centroid_distance)
                .collect::<Vec<f32>>();

            // Choose the next centroid with probability proportional
            // to the squared distance.
            let threshold = rng.gen::<f32>() * distances.iter().sum::<f32>();
            let mut cumulative_sum = 0.0;

            for (i, distance) in distances.iter().enumerate() {
                cumulative_sum += distance;
                if cumulative_sum >= threshold {
                    centroids.push(vectors[i].clone());
                    break;
                }
            }
        }

        centroids
    }

    fn update_centroids(&self, vectors: Vectors) -> Vec<Vector> {
        let dimension = vectors[0].len();
        let mut centroids = vec![vec![0.0; dimension]; self.n_clusters];
        let mut cluster_count = vec![0; self.n_clusters];

        // Sum up vectors assigned to the cluster into the centroid.
        for (i, cluster_id) in self.assignments.iter().enumerate() {
            let cluster_id = *cluster_id;
            cluster_count[cluster_id] += 1;
            centroids[cluster_id] = centroids[cluster_id]
                .iter()
                .zip(vectors[i].as_slice().iter())
                .map(|(a, b)| a + b)
                .collect();
        }

        // Divide the sum by the number of vectors in the cluster.
        for i in 0..self.n_clusters {
            // If the cluster is empty, reinitialize the centroid.
            if cluster_count[i] == 0 {
                let mut rng = rand::thread_rng();
                centroids[i] = vectors.choose(&mut rng).unwrap().to_vec();
                continue;
            }

            centroids[i] = centroids[i]
                .iter()
                .map(|x| x / cluster_count[i] as f32)
                .collect();
        }

        centroids.into_par_iter().map(|centroid| centroid.into()).collect()
    }

    /// Create cluster assignments for the vectors.
    fn assign_clusters(&self, vectors: Vectors) -> Vec<ClusterIndex> {
        vectors
            .par_iter()
            .map(|vector| self.find_nearest_centroid(vector))
            .collect()
    }

    /// Find the index of the nearest centroid from a vector.
    pub fn find_nearest_centroid(&self, vector: &Vector) -> ClusterIndex {
        self.centroids
            .par_iter()
            .enumerate()
            .map(|(i, centroid)| (i, self.metric.distance(vector, centroid)))
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(id, _)| id)
            .unwrap()
    }

    /// Returns index-mapped cluster assignment for each data point.
    ///
    /// The index corresponds to the data point index and the value corresponds
    /// to the cluster index. For example, given the following assignments:
    ///
    /// ```text
    /// [0, 1, 0, 1, 2]
    /// ```
    ///
    /// This means:
    /// - Point 0 and 2 are assigned to cluster 0.
    /// - Point 1 and 3 are assigned to cluster 1.
    /// - Point 4 is assigned to cluster 2.
    ///
    pub fn assignments(&self) -> &[ClusterIndex] {
        &self.assignments
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
    fn test_kmeans_fit_1_to_1() {
        evaluate_kmeans(1, generate_vectors(1));
    }

    #[test]
    fn test_kmeans_fit_10_to_5() {
        evaluate_kmeans(5, generate_vectors(10));
    }

    #[test]
    fn test_kmeans_fit_100_to_10() {
        evaluate_kmeans(10, generate_vectors(100));
    }

    fn evaluate_kmeans(n_cluster: usize, vectors: Vec<Vector>) {
        let vectors: Vectors = {
            let vectors_ref: Vec<&Vector> = vectors.iter().collect();
            Rc::from(vectors_ref.as_slice())
        };

        let mut kmeans = KMeans::new(n_cluster);
        kmeans.fit(vectors.clone()).unwrap();
        assert_eq!(kmeans.centroids().len(), n_cluster);

        let mut correct_count = 0;
        for (i, clusted_id) in kmeans.assignments().iter().enumerate() {
            let vector = vectors[i];
            let nearest_centroid = kmeans.find_nearest_centroid(vector);
            if clusted_id == &nearest_centroid {
                correct_count += 1;
            }
        }

        let accuracy = correct_count as f32 / vectors.len() as f32;
        assert!(accuracy > 0.99);
    }

    fn generate_vectors(n: usize) -> Vec<Vector> {
        (0..n).map(|i| Vector::from(vec![i as f32; 3])).collect()
    }
}
