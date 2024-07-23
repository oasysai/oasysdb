use crate::types::record::Vector;
use serde::{Deserialize, Serialize};
use simsimd::SpatialSimilarity;

/// Metric used to compare the distance between vectors in the index.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize, Clone, Copy, Hash)]
pub enum DistanceMetric {
    /// Squared [Euclidean distance](https://www.geeksforgeeks.org/euclidean-distance)
    ///
    /// The squared Euclidean distance is used to avoid the square
    /// root operation thus making the computation slightly faster.
    #[default]
    Euclidean,
    /// Cosine distance (1 - cosine similarity):
    /// [Cosine similarity](https://www.geeksforgeeks.org/cosine-similarity/)
    Cosine,
}

impl DistanceMetric {
    /// Computes the distance between two vectors.
    pub fn distance(&self, a: &Vector, b: &Vector) -> f32 {
        let a = &a.to_vec();
        let b = &b.to_vec();

        let dist = match self {
            DistanceMetric::Euclidean => f32::sqeuclidean(a, b),
            DistanceMetric::Cosine => f32::cosine(a, b),
        };

        // Distances of 0 is the best distance. So, we return a large
        // value for invalid values to make sure it is not selected.
        if dist.is_none()
            || dist.unwrap().is_nan()
            || dist.unwrap().is_infinite()
        {
            return f32::MAX;
        }

        dist.unwrap() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_metric() {
        let a = Vector::from(vec![1.0, 3.0, 5.0]);
        let b = Vector::from(vec![2.0, 4.0, 6.0]);

        let metric = DistanceMetric::Euclidean;
        let dist = metric.distance(&a, &b);
        assert_eq!(dist, 3.0);

        let metric = DistanceMetric::Cosine;
        let dist = metric.distance(&a, &b);
        assert!(dist <= 0.01);
    }
}
