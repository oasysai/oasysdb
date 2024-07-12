use super::*;
use std::collections::BinaryHeap;

/// Brute force index implementation.
///
/// This index stores all records in memory and performs a linear search
/// for the nearest neighbors. It is great for small datasets of less than
/// 10,000 records due to perfect recall and precision.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexBruteForce {
    config: SourceConfig,
    metric: DistanceMetric,
    metadata: IndexMetadata,
    data: HashMap<RecordID, Record>,
}

impl IndexOps for IndexBruteForce {
    fn new(config: SourceConfig, metric: DistanceMetric) -> Self {
        Self {
            config,
            metric,
            metadata: IndexMetadata::default(),
            data: HashMap::new(),
        }
    }

    fn config(&self) -> &SourceConfig {
        &self.config
    }

    fn metric(&self) -> &DistanceMetric {
        &self.metric
    }

    fn metadata(&self) -> &IndexMetadata {
        &self.metadata
    }
}

impl VectorIndex for IndexBruteForce {
    fn fit(&mut self, records: HashMap<RecordID, Record>) -> Result<(), Error> {
        if records.is_empty() {
            return Ok(());
        }

        self.metadata.last_inserted = records.keys().max().copied();
        self.metadata.count += records.len();
        self.data.par_extend(records);

        Ok(())
    }

    /// Refitting doesn't do anything for the brute force index
    /// as incremental insertion or deletion will directly update
    /// the data store accordingly guaranteeing the index optimal state.
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
    ) -> Result<Vec<SearchResult>, Error> {
        let mut results = BinaryHeap::new();
        for (id, record) in &self.data {
            let distance = self.metric.distance(&record.vector, &query);
            let data = record.data.clone();
            results.push(SearchResult { id: *id, distance, data });

            if results.len() > k {
                results.pop();
            }
        }

        Ok(results.into_sorted_vec())
    }

    fn search_with_filters(
        &self,
        query: Vector,
        k: usize,
        filters: Filters,
    ) -> Result<Vec<SearchResult>, Error> {
        if filters == Filters::NONE {
            return self.search(query, k);
        }

        let mut results = BinaryHeap::new();
        for (id, record) in &self.data {
            if filters.apply(&record.data) {
                let distance = self.metric.distance(&record.vector, &query);
                let data = record.data.clone();
                results.push(SearchResult { id: *id, distance, data });

                if results.len() > k {
                    results.pop();
                }
            }
        }

        Ok(results.into_sorted_vec())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bruteforce_index() {
        let config = SourceConfig::default();
        let metric = DistanceMetric::Euclidean;
        let mut index = IndexBruteForce::new(config, metric);
        index_tests::populate_index(&mut index);
        index_tests::test_search(&index);
        index_tests::test_search_with_filters(&index);
    }
}
