use serde::{Deserialize, Serialize};

use super::{err::Error, Vector};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Distance {
    Dot,
    Euclidean,
    Cosine,
}

impl Distance {
    pub fn from(distance: &str) -> Result<Self, Error> {
        match distance {
            "dot" => Ok(Distance::Dot),
            "euclidean" => Ok(Distance::Euclidean),
            "cosine" => Ok(Distance::Cosine),
            _ => Err("Distance not supported".into()),
        }
    }

    pub fn calculate(&self, a: &Vector, b: &Vector) -> f32 {
        assert_eq!(a.0.len(), b.0.len());

        match self {
            Distance::Dot => calculate_dot(a, b),
            Distance::Euclidean => calculate_euclidean(a, b),
            Distance::Cosine => calculate_cosine(a, b),
        }
    }
}

fn calculate_dot(a: &Vector, b: &Vector) -> f32 {
    a.0.iter().zip(b.0.iter()).map(|(x, y)| x * y).sum::<f32>()
}
fn calculate_cosine(a: &Vector, b: &Vector) -> f32 {
    let dot_product: f32 = a.0.iter().zip(b.0.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.0.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.0.iter().map(|y| y.powi(2)).sum::<f32>().sqrt();

    dot_product / (magnitude_a * magnitude_b)
}
fn calculate_euclidean(a: &Vector, b: &Vector) -> f32 {
    a.0.iter().zip(b.0.iter()).map(|(a, b)| (a - b).powi(2)).sum::<f32>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_calcultation() {
        let v = Vector::from(vec![1.0, 3.0, 5.0, 7.0]);
        let n = Vector::from(vec![11.0, 13.0, 17.0, 19.0]);
        let dot_product: f32 =
            v.0.iter().zip(n.0.iter()).map(|(vi, ni)| vi * ni).sum();
        let euclidean_distance: f32 =
            v.0.iter()
                .zip(n.0.iter())
                .map(|(vi, ni)| (vi - ni).powi(2))
                .sum::<f32>()
                .sqrt();
        let magnitude_v: f32 =
            v.0.iter().map(|vi| vi.powi(2)).sum::<f32>().sqrt();
        let magnitude_n: f32 =
            n.0.iter().map(|ni| ni.powi(2)).sum::<f32>().sqrt();
        let cosine_similarity: f32 = dot_product / (magnitude_v * magnitude_n);

        let test_cases: Vec<(String, f32)> = vec![
            ("dot".into(), dot_product),
            ("euclidean".into(), euclidean_distance),
            ("cosine".into(), cosine_similarity),
        ];

        for (distance, right) in test_cases {
            let dist_fun = Distance::from(&distance).unwrap();
            let left = dist_fun.calculate(&v, &n);
            assert_eq!(
                left, right,
                "dist: {}, left: {}, right {}",
                distance, left, right
            );
        }
    }
}
