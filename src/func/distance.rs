use super::*;

/// The distance function used for similarity calculations.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Distance {
    /// Euclidean distance function.
    Euclidean,
    /// Cosine distance function (1 - Cosine similarity).
    Cosine,
}

impl Distance {
    /// Creates a new distance function from a string.
    /// Available options:
    /// * `euclidean`: Euclidean distance function.
    /// * `cosine`: Cosine similarity function.
    pub fn from(distance: &str) -> Result<Self, Error> {
        match distance {
            "euclidean" => Ok(Distance::Euclidean),
            "cosine" => Ok(Distance::Cosine),
            _ => Err(Error::invalid_distance()),
        }
    }

    /// Calculates the distance between two vectors.
    pub fn calculate(&self, a: &Vector, b: &Vector) -> f32 {
        assert_eq!(a.0.len(), b.0.len());
        match self {
            Distance::Euclidean => Distance::euclidean(a, b),
            Distance::Cosine => Distance::cosine(a, b),
        }
    }

    // List additional distance functions below.

    fn cosine(a: &Vector, b: &Vector) -> f32 {
        f32::cosine(&a.0, &b.0).unwrap() as f32
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
            Distance::Euclidean => "euclidean".into_py(py),
            Distance::Cosine => "cosine".into_py(py),
        }
    }
}
