use super::*;

/// Flat index implementation.
///
/// This index stores all records in memory and performs a linear search
/// for the nearest neighbors. It is great for small datasets of less than
/// 10,000 records due to perfect recall and precision.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexFlat {
    params: ParamsFlat,
    metadata: IndexMetadata,
    data: HashMap<RecordID, Record>,
}

impl IndexOps for IndexFlat {
    fn new(params: impl IndexParams) -> Result<IndexFlat, Error> {
        let index = IndexFlat {
            params: downcast_params(params)?,
            metadata: IndexMetadata::default(),
            data: HashMap::new(),
        };

        Ok(index)
    }
}

impl VectorIndex for IndexFlat {
    fn metric(&self) -> &DistanceMetric {
        &self.params.metric
    }

    fn metadata(&self) -> &IndexMetadata {
        &self.metadata
    }

    fn build(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error> {
        self.metadata.built = true;
        self.insert(records)
    }

    fn insert(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error> {
        if records.is_empty() {
            return Ok(());
        }

        self.metadata.last_inserted = records.keys().max().copied();
        self.data.par_extend(records);
        Ok(())
    }

    fn delete(&mut self, ids: Vec<RecordID>) -> Result<(), Error> {
        self.data.retain(|id, _| !ids.contains(id));
        Ok(())
    }

    fn search(
        &self,
        query: Vector,
        k: usize,
        filters: Filters,
    ) -> Result<Vec<SearchResult>, Error> {
        let mut results = BinaryHeap::new();
        for (id, record) in &self.data {
            // Skip records that don't pass the filters.
            if !filters.apply(&record.data) {
                continue;
            }

            let distance = self.metric().distance(&record.vector, &query);
            let data = record.data.clone();
            results.push(SearchResult { id: *id, distance, data });

            if results.len() > k {
                results.pop();
            }
        }

        Ok(results.into_sorted_vec())
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Parameters for IndexFlat.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParamsFlat {
    /// Formula used to calculate the distance between vectors.
    pub metric: DistanceMetric,
}

impl IndexParams for ParamsFlat {
    fn metric(&self) -> &DistanceMetric {
        &self.metric
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_index() {
        let params = ParamsFlat::default();
        let mut index = IndexFlat::new(params).unwrap();

        index_tests::populate_index(&mut index);
        index_tests::test_basic_search(&index);
        index_tests::test_advanced_search(&index);
    }
}
