use super::*;

/// Algorithm options used to index and search vectors.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum IndexAlgorithm {
    BruteForce, // -> IndexBruteForce
}

impl IndexAlgorithm {
    /// Initializes a new index based on the algorithm and configuration.
    pub(crate) fn initialize(
        &self,
        config: SourceConfig,
        metric: DistanceMetric,
    ) -> Box<dyn VectorIndex> {
        let index = match self {
            IndexAlgorithm::BruteForce => IndexBruteForce::new(config, metric),
        };

        Box::new(index)
    }

    /// Persists the index to a file based on the algorithm.
    /// - `path`: Path to the file where the index will be stored.
    /// - `index`: Index to persist as a trait object.
    pub(crate) fn persist_index(
        &self,
        path: impl AsRef<Path>,
        index: Box<dyn VectorIndex>,
    ) -> Result<(), Error> {
        match self {
            IndexAlgorithm::BruteForce => {
                Self::_persist_index::<IndexBruteForce>(path, index)
            }
        }
    }

    fn _persist_index<T: VectorIndex + IndexOps + 'static>(
        path: impl AsRef<Path>,
        index: Box<dyn VectorIndex>,
    ) -> Result<(), Error> {
        let index = index.as_any().downcast_ref::<T>().ok_or_else(|| {
            let code = ErrorCode::InternalError;
            let message = "Failed to downcast index to concrete type.";
            Error::new(code, message)
        })?;

        index.persist(path)?;
        Ok(())
    }
}
