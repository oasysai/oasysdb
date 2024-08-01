#![allow(unreachable_code)]

use crate::types::record::Vector;
use serde::{Deserialize, Serialize};

#[cfg(feature = "simd")]
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
            DistanceMetric::Euclidean => Self::sqeuclidean(a, b),
            DistanceMetric::Cosine => Self::cosine(a, b),
        };

        dist.unwrap() as f32
    }

    fn sqeuclidean(a: &[f32], b: &[f32]) -> Option<f64> {
        #[cfg(feature = "simd")]
        return f32::sqeuclidean(a, b);

        let dist = a
            .iter()
            .zip(b.iter())
            .map(|(a, b)| (a - b).powi(2) as f64)
            .sum::<f64>();

        Some(dist)
    }

    fn cosine(a: &[f32], b: &[f32]) -> Option<f64> {
        #[cfg(feature = "simd")]
        return f32::cosine(a, b);

        let dot = a.iter().zip(b.iter()).map(|(a, b)| a * b).sum::<f32>();
        let norm_a = a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();

        let dist = 1.0 - dot / (norm_a * norm_b);
        Some(dist as f64)
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
