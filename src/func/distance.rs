use super::*;

/// The distance function used for similarity calculations.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Distance {
    /// Dot product function.
    Dot,
    /// Euclidean distance function.
    Euclidean,
    /// Cosine similarity function.
    Cosine,
    /// Cosine function for normalized vectors.
    NormCosine,
}

impl Distance {
    /// Creates a new distance function from a string.
    /// Available options:
    /// * `dot`: Dot product function.
    /// * `euclidean`: Euclidean distance function.
    /// * `cosine`: Cosine similarity function.
    /// * `norm_cosine`: Cosine function for normalized vectors.
    pub fn from(distance: &str) -> Result<Self, Error> {
        match distance {
            "dot" => Ok(Distance::Dot),
            "euclidean" => Ok(Distance::Euclidean),
            "cosine" => Ok(Distance::Cosine),
            "norm_cosine" => Ok(Distance::NormCosine),
            _ => Err(Error::invalid_distance()),
        }
    }

    /// Calculates the distance between two vectors.
    pub fn calculate(&self, a: &Vector, b: &Vector) -> f32 {
        assert_eq!(a.0.len(), b.0.len());
        match self {
            Distance::Dot => Distance::dot(a, b),
            Distance::Euclidean => Distance::euclidean(a, b),
            Distance::Cosine => Distance::cosine(a, b),
            Distance::NormCosine => Distance::dot(a, b),
        }
    }

    // List additional distance functions below.

    fn dot(a: &Vector, b: &Vector) -> f32 {
        f32::dot(&a.0, &b.0).unwrap() as f32
    }

    fn cosine(a: &Vector, b: &Vector) -> f32 {
        let dot = Self::dot(a, b);
        let ma = a.0.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
        let mb = b.0.iter().map(|y| y.powi(2)).sum::<f32>().sqrt();
        dot / (ma * mb)
    }

    fn euclidean(a: &Vector, b: &Vector) -> f32 {
        let sq = f32::sqeuclidean(&a.0, &b.0).unwrap() as f32;
        sq.sqrt()
    }
}

#[cfg(feature = "py")]
impl From<&PyAny> for Distance {
    fn from(distance: &PyAny) -> Self {
        let distance = distance.str().unwrap().to_string();
        Distance::from(&distance).unwrap()
    }
}

#[cfg(feature = "py")]
impl IntoPy<Py<PyAny>> for Distance {
    fn into_py(self, py: Python) -> Py<PyAny> {
        match self {
            Distance::Dot => "dot".into_py(py),
            Distance::Euclidean => "euclidean".into_py(py),
            Distance::Cosine => "cosine".into_py(py),
            Distance::NormCosine => "norm_cosine".into_py(py),
        }
    }
}
