use super::*;
use crate::utils::kmeans::{KMeans, Vectors};
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexIVFPQ {
    config: SourceConfig,
    params: ParamsIVFPQ,
    metadata: IndexMetadata,
    data: HashMap<RecordID, RecordPQ>,

    // IVFPQ specific data structures.
    centroids: Vec<Vector>,
    clusters: Vec<Vec<RecordID>>,
    codebook: Vec<Vec<Vector>>,
}

impl IndexIVFPQ {
    fn build(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error> {
        if records.is_empty() {
            return Ok(());
        }

        let vectors = records
            .values()
            .map(|record| &record.vector)
            .collect::<Vec<&Vector>>();

        let vectors: Vectors = Rc::from(vectors.as_slice());
        self.create_codebook(vectors.clone());

        let (centroids, assignments) = {
            let mut kmeans = KMeans::new(
                self.params.centroids,
                self.params.num_iterations,
                self.metric().to_owned(),
            );

            kmeans.fit(vectors.clone());
            (kmeans.centroids().to_vec(), kmeans.assignments().to_vec())
        };

        self.centroids = centroids;
        self.clusters = {
            let mut clusters = vec![vec![]; self.params.centroids];
            let ids = records.keys().collect::<Vec<&RecordID>>();
            for (i, &cluster) in assignments.iter().enumerate() {
                clusters[cluster.0 as usize].push(ids[i].to_owned());
            }

            clusters
        };

        self.metadata.count = records.len();
        self.metadata.last_inserted = records.keys().max().copied();

        self.data = records
            .into_iter()
            .map(|(id, record)| {
                let vector = self.quantize_vector(&record.vector);
                let data = record.data;
                (id, RecordPQ { vector, data })
            })
            .collect();

        Ok(())
    }

    fn insert(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error> {
        if records.is_empty() {
            return Ok(());
        }

        let assignments = records
            .values()
            .map(|record| self.find_nearest_centroids(&record.vector, 1)[0])
            .collect::<Vec<usize>>();

        let ids: Vec<&RecordID> = records.keys().collect();
        for (i, cluster_id) in assignments.iter().enumerate() {
            self.clusters[*cluster_id].push(ids[i].to_owned());
        }

        self.metadata.count += records.len();
        self.metadata.last_inserted = records.keys().max().copied();

        let records: HashMap<RecordID, RecordPQ> = records
            .into_par_iter()
            .map(|(id, record)| {
                let vector = self.quantize_vector(&record.vector);
                let data = record.data;
                (id, RecordPQ { vector, data })
            })
            .collect();

        self.data.par_extend(records);
        Ok(())
    }

    fn create_codebook(&mut self, vectors: Vectors) {
        for i in 0..self.params.sub_dimension {
            let mut subvectors = Vec::new();
            for vector in vectors.iter() {
                let subvector = self.get_subvector(i.into(), vector);
                subvectors.push(subvector);
            }

            let centroids = {
                let mut kmeans = KMeans::new(
                    self.params.sub_centroids as usize,
                    self.params.num_iterations,
                    self.params.metric,
                );

                let subvectors: Vec<&Vector> = subvectors.iter().collect();
                kmeans.fit(Rc::from(subvectors));
                kmeans.centroids().to_vec()
            };

            self.codebook[i as usize] = centroids
                .par_iter()
                .map(|centroid| centroid.clone().into())
                .collect();
        }
    }

    fn find_nearest_centroids(&self, vector: &Vector, k: usize) -> Vec<usize> {
        let mut distances: Vec<(usize, f32)> = self
            .centroids
            .iter()
            .enumerate()
            .map(|(i, center)| (i, self.metric().distance(center, vector)))
            .collect();

        distances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        distances.into_iter().take(k).map(|(i, _)| i).collect()
    }

    fn find_nearest_code(
        &self,
        part_index: usize,
        subvector: &Vector,
    ) -> usize {
        self.codebook[part_index]
            .iter()
            .map(|centroid| self.metric().distance(centroid, subvector))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap_or_default()
            .0
    }

    fn quantize_vector(&self, vector: &Vector) -> VectorPQ {
        let sub_dimension = self.params.sub_dimension as usize;
        let mut pq = Vec::with_capacity(sub_dimension);

        for i in 0..sub_dimension {
            let subvector = self.get_subvector(i, vector);
            let centroid_id = self.find_nearest_code(i, &subvector);
            pq.push(centroid_id as u8);
        }

        pq.into()
    }

    fn dequantize_vector(&self, vector_pq: &VectorPQ) -> Vector {
        let mut vector = vec![];
        for (i, centroid_id) in vector_pq.0.iter().enumerate() {
            let centroid = &self.codebook[i][*centroid_id as usize];
            vector.extend(centroid.to_vec());
        }

        vector.into()
    }

    fn get_subvector(&self, part_index: usize, vector: &Vector) -> Vector {
        let dim = vector.len() / self.params.sub_dimension as usize;
        let start = part_index as usize * dim;
        let end = (part_index + 1) as usize * dim;
        let subvector = vector.0[start..end].to_vec();
        Vector(subvector.into_boxed_slice())
    }
}

impl IndexOps for IndexIVFPQ {
    fn new(
        config: SourceConfig,
        params: impl IndexParams,
    ) -> Result<Self, Error> {
        let params = downcast_params::<ParamsIVFPQ>(params)?;
        let codebook = vec![vec![]; params.sub_dimension as usize];
        let clusters = vec![vec![]; params.centroids];

        let index = IndexIVFPQ {
            config,
            params,
            metadata: IndexMetadata::default(),
            data: HashMap::new(),

            centroids: vec![],
            clusters,
            codebook,
        };

        Ok(index)
    }
}

impl VectorIndex for IndexIVFPQ {
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
        match self.metadata.count {
            0 => self.build(records),
            _ => self.insert(records),
        }
    }

    fn refit(&mut self) -> Result<(), Error> {
        self.data.retain(|id, _| !self.metadata.hidden.contains(id));

        let records = self
            .data
            .par_iter()
            .map(|(id, record)| {
                let vector = self.dequantize_vector(&record.vector);
                let data = record.data.clone();
                (*id, Record { vector, data })
            })
            .collect();

        self.build(records)
    }

    fn search(
        &self,
        query: Vector,
        k: usize,
        filters: Filters,
    ) -> Result<Vec<SearchResult>, Error> {
        let nearest_centroids = {
            let nprobes = self.params.num_probes as usize;
            self.find_nearest_centroids(&query, nprobes)
        };

        let mut results = BinaryHeap::new();
        for centroid_id in nearest_centroids {
            let cluster = &self.clusters[centroid_id];
            for &record_id in cluster {
                let record = self.data.get(&record_id).unwrap();
                let data = record.data.clone();
                if !filters.apply(&data) {
                    continue;
                }

                let vector = self.dequantize_vector(&record.vector);
                let distance = self.metric().distance(&vector, &query);
                results.push(SearchResult { id: record_id, distance, data });

                if results.len() > k {
                    results.pop();
                }
            }
        }

        Ok(results.into_sorted_vec())
    }

    fn hide(&mut self, record_ids: Vec<RecordID>) -> Result<(), Error> {
        self.metadata.hidden.extend(record_ids);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsIVFPQ {
    /// Number of centroids in the IVF.
    pub centroids: usize,
    /// Number of iterations to run the KMeans algorithm.
    pub num_iterations: usize,
    /// Number of centroids in the PQ sub-space.
    pub sub_centroids: u8,
    /// Dimension of the vector after PQ encoding.
    pub sub_dimension: u8,
    /// Number of clusters to explore during search.
    pub num_probes: u8,
    /// Metric used to compute the distance between vectors.
    pub metric: DistanceMetric,
}

impl Default for ParamsIVFPQ {
    fn default() -> Self {
        Self {
            num_iterations: 100,
            centroids: 256,
            sub_centroids: 32,
            sub_dimension: 16,
            num_probes: 4,
            metric: DistanceMetric::Euclidean,
        }
    }
}

impl IndexParams for ParamsIVFPQ {
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
    fn test_product_quantization() {
        let data: Vec<Vector> = vec![
            vec![1.0, 2.0, 3.0, 4.0].into(),
            vec![5.0, 6.0, 7.0, 8.0].into(),
            vec![9.0, 10.0, 11.0, 12.0].into(),
            vec![13.0, 14.0, 15.0, 16.0].into(),
        ];

        let vectors: Vectors = {
            let data = data.iter().collect::<Vec<&Vector>>();
            Rc::from(data.as_slice())
        };

        let params = ParamsIVFPQ {
            num_iterations: 10,
            sub_centroids: 8,
            sub_dimension: 2,
            ..Default::default()
        };

        let mut index = create_test_index(params);
        index.create_codebook(vectors);

        let encoded = index.quantize_vector(&data[0]);
        let decoded = index.dequantize_vector(&encoded);
        assert_eq!(decoded.to_vec(), data[0].to_vec());
    }

    #[test]
    fn test_ivfpq_index() {
        let params = ParamsIVFPQ {
            centroids: 5,
            num_iterations: 20,
            ..Default::default()
        };

        let mut index = create_test_index(params);
        index_tests::populate_index(&mut index);
        index_tests::test_basic_search(&index);
        index_tests::test_advanced_search(&index);
    }

    fn create_test_index(params: ParamsIVFPQ) -> IndexIVFPQ {
        let config = SourceConfig::default();
        IndexIVFPQ::new(config, params).unwrap()
    }
}
