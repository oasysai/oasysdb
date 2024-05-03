use crate::prelude::*;

/// Trait for embedding models to easily generate vectors.
pub trait EmbeddingModel {
    /// Returns the model ID: `provider-name/model-name`
    /// - `provider-name`: Model provider like openai, google, etc.
    /// - `model-name`: Model name like gpt-3, bert, etc.
    fn id(&self) -> &str;

    /// Creates a vector embedding from the given content.
    /// - `content`: Text or content URL to generate the vector.
    fn create_vector(&self, content: &str) -> Result<Vector, Error>;

    /// Creates a vector record from content and data.
    /// - `content`: Text or content URL to generate the vector.
    /// - `data`: Metadata to associate with the vector.
    fn create_record(
        &self,
        content: &str,
        data: &Metadata,
    ) -> Result<Record, Error>;
}
