use super::*;
use array::downcast_array;
use regex::Regex;

impl Collection {
    /// Validates the name of collections or fields.
    pub fn validate_name(name: &str) -> Result<(), Error> {
        if name.is_empty() {
            let code = ErrorCode::ClientError;
            let message = "Name cannot be empty";
            return Err(Error::new(&code, message));
        }

        // We only allow lowercase letters and underscores in the names.
        // Also, we can unwrap here because the regex pattern is hardcoded.
        let re = Regex::new(r"^[a-z0-9_]+$").unwrap();
        if !re.is_match(name) {
            return Err(Error::new(
                &ErrorCode::ClientError,
                "Name must be lowercase letters with underscores.",
            ));
        }

        Ok(())
    }

    /// Validates the vectors given a column array consisting of vectors.
    /// This ensures that all vectors provided have the same dimension.
    pub fn validate_vectors(
        &self,
        vectors: &Arc<dyn Array>,
        dimension: usize,
    ) -> Result<(), Error> {
        let vector_array: ListArray = downcast_array(vectors.as_ref());

        let is_dimension_mismatch = |array: Arc<dyn Array>| {
            let vector: Float32Array = downcast_array(array.as_ref());
            vector.len() != dimension
        };

        let dimension_mismatch = vector_array.iter().any(|array| match array {
            Some(array) => is_dimension_mismatch(array),
            None => true,
        });

        if dimension_mismatch {
            let code = ErrorCode::ClientError;
            let message = "Vectors must have the same dimension.";
            return Err(Error::new(&code, message));
        }

        Ok(())
    }
}
