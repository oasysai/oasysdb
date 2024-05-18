use super::*;

/// The filter operations for the collection metadata.
pub enum Filter {
    /// Text data equals to the filter value.
    TextEqual(String),
    /// Text data includes the filter value.
    TextInclude(String),
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
        if let Filter::TextEqual(value) = self {
            if let Metadata::Text(text) = metadata {
                return text == value;
            }
        }

        if let Filter::TextInclude(value) = self {
            if let Metadata::Text(text) = metadata {
                return text.contains(value);
            }
        }

        false
    }
}
