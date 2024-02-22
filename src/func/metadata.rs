use super::*;

/// The metadata associated with a vector record.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Metadata {
    /// A piece of text like article title or description.
    Text(String),
    /// An integer number such as external IDs.
    Integer(usize),
    /// A float number to represent something like a score.
    Float(f32),
    /// An array containing any type of metadata.
    Array(Vec<Metadata>),
    /// A map of string and metadata pairs. The most common type.
    Object(HashMap<String, Metadata>),
}

impl From<usize> for Metadata {
    fn from(value: usize) -> Self {
        Metadata::Integer(value)
    }
}

impl From<f32> for Metadata {
    fn from(value: f32) -> Self {
        Metadata::Float(value)
    }
}

impl From<String> for Metadata {
    fn from(value: String) -> Self {
        Metadata::Text(value)
    }
}

impl From<&str> for Metadata {
    fn from(value: &str) -> Self {
        Metadata::Text(value.to_string())
    }
}

impl<T> From<Vec<T>> for Metadata
where
    Metadata: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        let arr = value.into_iter().map(|v| v.into()).collect();
        Metadata::Array(arr)
    }
}

impl<T> From<HashMap<String, T>> for Metadata
where
    Metadata: From<T>,
{
    fn from(value: HashMap<String, T>) -> Self {
        let iter = value.into_iter();
        let obj = iter.map(|(k, v)| (k, v.into())).collect();
        Metadata::Object(obj)
    }
}

impl<T> From<HashMap<&str, T>> for Metadata
where
    Metadata: From<T>,
{
    fn from(value: HashMap<&str, T>) -> Self {
        let iter = value.into_iter();
        let obj = iter.map(|(k, v)| (k.into(), v.into())).collect();
        Metadata::Object(obj)
    }
}
