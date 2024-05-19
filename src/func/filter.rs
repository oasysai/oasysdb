use super::*;

/// The filter operations for the collection metadata.
#[derive(Clone, Debug)]
pub enum Filter {
    /// Text data includes the filter value.
    Text(String),
    /// Object data has all key-value pair that matches the filter.
    Object(HashMap<String, Filter>),
}

impl Filter {
    /// Filters the data and returns the matching vector IDs.
    pub fn get_vector_ids(
        &self,
        data: &HashMap<VectorID, Metadata>,
    ) -> Vec<VectorID> {
        data.par_iter()
            .filter_map(|(id, data)| match self.match_metadata(data) {
                true => Some(*id),
                false => None,
            })
            .collect()
    }

    fn match_metadata(&self, metadata: &Metadata) -> bool {
        if let Filter::Text(value) = self {
            if let Metadata::Text(text) = metadata {
                return text.contains(value);
            }
        }

        if let Filter::Object(filters) = self {
            if let Metadata::Object(object) = metadata {
                return self.match_metadata_object(filters, object);
            }
        }

        false
    }

    fn match_metadata_object(
        &self,
        filters: &HashMap<String, Filter>,
        object: &HashMap<String, Metadata>,
    ) -> bool {
        for (key, filter) in filters {
            let data = object.get(key);

            // If the key doesn't have a value, it doesn't match.
            if data.is_none() {
                return false;
            }

            // If the value doesn't match the filter, it doesn't match.
            if !filter.match_metadata(data.unwrap()) {
                return false;
            }
        }

        // Only return true if all key-value pairs match.
        true
    }
}

// Interoperability from primitive types to Filter.

impl From<&str> for Filter {
    fn from(value: &str) -> Self {
        Filter::Text(value.to_string())
    }
}

impl From<String> for Filter {
    fn from(value: String) -> Self {
        Filter::Text(value)
    }
}

impl<T> From<HashMap<String, T>> for Filter
where
    Filter: From<T>,
{
    fn from(value: HashMap<String, T>) -> Self {
        let iter = value.into_iter();
        let object = iter.map(|(k, v)| (k, v.into())).collect();
        Filter::Object(object)
    }
}

impl<T> From<HashMap<&str, T>> for Filter
where
    Filter: From<T>,
{
    fn from(value: HashMap<&str, T>) -> Self {
        let iter = value.into_iter();
        let object = iter.map(|(k, v)| (k.into(), v.into())).collect();
        Filter::Object(object)
    }
}
