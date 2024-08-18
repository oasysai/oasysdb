use super::*;
use simsimd::SpatialSimilarity;

/// Distance formula for vector similarity calculations.
///
/// ### Euclidean
/// We use the squared Euclidean distance instead for a slight performance
/// boost since we only use the distance for comparison.
///
/// ### Cosine
/// We use cosine distance instead of cosine similarity to be consistent with
/// other distance metrics where a lower value indicates a closer match.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Metric {
    Euclidean,
    Cosine,
}

impl Metric {
    /// Calculate the distance between two vectors.
    pub fn distance(&self, a: &Vector, b: &Vector) -> f32 {
        let (a, b) = (a.as_slice(), b.as_slice());
        let d = match self {
            Metric::Euclidean => f32::sqeuclidean(a, b),
            Metric::Cosine => f32::cosine(a, b),
        };

        // TODO: Handle non-numerical values like NaN.
        d.unwrap() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let a = Vector::from(vec![1.0, 2.0, 3.0]);
        let b = Vector::from(vec![4.0, 5.0, 6.0]);

        let euclidean = Metric::Euclidean.distance(&a, &b);
        let cosine = Metric::Cosine.distance(&a, &b);

        assert_eq!(euclidean, 27.0);
        assert_eq!(cosine.round(), 0.0);
    }
}
