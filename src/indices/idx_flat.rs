use super::*;

/// Flat index implementation.
///
/// This index stores all records in memory and performs a linear search
/// for the nearest neighbors. It is great for small datasets of less than
/// 10,000 records due to perfect recall and precision.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexFlat {
    config: SourceConfig,
    params: ParamsFlat,
    metadata: IndexMetadata,
    data: HashMap<RecordID, Record>,
}

impl IndexOps for IndexFlat {
    fn new(
        config: SourceConfig,
        params: impl IndexParams,
    ) -> Result<IndexFlat, Error> {
        let index = IndexFlat {
            config,
            params: downcast_params(params)?,
            metadata: IndexMetadata::default(),
            data: HashMap::new(),
        };

        Ok(index)
    }
}

impl VectorIndex for IndexFlat {
    fn config(&self) -> &SourceConfig {
        &self.config
    }

    fn metric(&self) -> &DistanceMetric {
        &self.params.metric
    }

    fn metadata(&self) -> &IndexMetadata {
        &self.metadata
    }

    fn fit(&mut self, records: HashMap<RecordID, Record>) -> Result<(), Error> {
        if records.is_empty() {
            return Ok(());
        }

        self.metadata.last_inserted = records.keys().max().copied();
        self.metadata.count += records.len();
        self.data.par_extend(records);

        Ok(())
    }

    /// Refitting doesn't do anything for the flat index as incremental
    /// insertion or deletion will directly update the data store
    /// accordingly which guarantee the optimal state of the index.
    fn refit(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// Removes records from the index data store.
    /// - `record_ids`: List of record IDs to remove from the index.
    fn hide(&mut self, record_ids: Vec<RecordID>) -> Result<(), Error> {
        if self.data.len() < record_ids.len() {
            return Ok(());
        }

        self.data.retain(|id, _| !record_ids.contains(id));
        self.metadata.count = self.data.len();
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Flat index parameters.
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
        let config = SourceConfig::default();
        let params = ParamsFlat::default();
        let mut index = IndexFlat::new(config, params).unwrap();

        index_tests::populate_index(&mut index);
        index_tests::test_basic_search(&index);
        index_tests::test_advanced_search(&index);
    }
}
