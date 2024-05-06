use super::*;

/// Embedding models provided by OpenAI.
pub struct OpenAI {
    /// OpenAI API key.
    pub api_key: String,
    /// Embedding model name.
    pub model: String,
    endpoint: String,
}

impl EmbeddingModel for OpenAI {
    fn id(&self) -> &str {
        let id = format!("openai/{}", self.model);
        Box::leak(id.into_boxed_str())
    }

    fn create_vector(&self, content: &str) -> Result<Vector, Error> {
        self.create_vector(content)
    }

    fn create_record(
        &self,
        content: &str,
        data: &Metadata,
    ) -> Result<Record, Error> {
        let vector = self.create_vector(content)?;
        let record = Record::new(&vector, data);
        Ok(record)
    }
}

impl OpenAI {
    /// Creates a new OpenAI embedding model instance.
    pub fn new(api_key: &str, model: &str) -> Self {
        let valid_models = [
            "text-embedding-3-large",
            "text-embedding-3-small",
            "text-embedding-ada-002",
        ];

        // Validate the model input.
        if !valid_models.contains(&model) {
            panic!("Unsupported embedding model: {model}");
        }

        let endpoint = "https://api.openai.com/v1/embeddings";

        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            endpoint: endpoint.to_string(),
        }
    }

    fn create_vector(&self, content: &str) -> Result<Vector, Error> {
        let bearer = format!("Bearer {}", self.api_key);

        // Create the request body for the API.
        // https://platform.openai.com/docs/api-reference/embeddings/create
        let body = json!({
            "input": content,
            "model": self.model,
        });

        let client = Client::new();
        let response = client
            .post(&self.endpoint)
            .header("authorization", bearer)
            .json(&body)
            .send()?;

        // Get the JSON response from the API.
        let json: Value = response.json()?;
        let embedding = &json["data"][0]["embedding"];
        let vector: Vec<f32> = serde_json::from_value(embedding.clone())?;

        Ok(Vector::from(vector))
    }

    /// Set custom endpoint for the OpenAI API.
    pub fn with_endpoint(&mut self, endpoint: &str) -> &mut Self {
        // Validate the endpoint URL.
        if !endpoint.starts_with("https://api.openai.com") {
            panic!("Invalid OpenAI API endpoint: {endpoint}");
        }

        self.endpoint = endpoint.to_string();
        self
    }
}
