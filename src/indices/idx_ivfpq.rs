use super::*;
use crate::utils::kmeans::{KMeans, Vectors};
use std::rc::Rc;

/// Inverted File index with Product Quantization.
///
/// This index is a composite index that combines the Inverted File
/// algorithm with Product Quantization to achieve a balance between
/// memory usage and search speed. It is a great choice for large
/// datasets with millions of records.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexIVFPQ {
    params: ParamsIVFPQ,
    metadata: IndexMetadata,
    data: HashMap<RecordID, RecordPQ>,

    // IVFPQ specific data structures.
    centroids: Vec<Vector>,
    clusters: Vec<Vec<RecordID>>,
    codebook: Vec<Vec<Vector>>,
}

impl IndexIVFPQ {
    /// Builds the index from scratch.
    /// - `records`: Dataset to build the index from.
    ///
    /// This method should only be called when the index is first
    /// initialized or when the index needs to be rebuilt from scratch
    /// because this method will overwrite the existing index data.
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

        // We use RC to avoid cloning the entire vector data as it
        // can be very large and expensive to clone.
        let vectors: Vectors = Rc::from(vectors.as_slice());
        self.create_codebook(vectors.clone());

        // Run KMeans to find the centroids for the IVF.
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
            // Put record IDs into their respective clusters based on the
            // assignments from the KMeans algorithm.
            let mut clusters = vec![vec![]; self.params.centroids];
            let ids = records.keys().collect::<Vec<&RecordID>>();
            for (i, &cluster) in assignments.iter().enumerate() {
                clusters[cluster.0 as usize].push(ids[i].to_owned());
            }

            clusters
        };

        self.metadata.count = records.len();
        self.metadata.last_inserted = records.keys().max().copied();

        // Store the quantized vectors instead of the original vectors.
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

    /// Inserts new records into the index incrementally.
    /// - `records`: New records to insert.
    fn insert(
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

        let assignments = vectors
            .par_iter()
            .map(|vector| self.find_nearest_centroids(vector, 1)[0])
            .collect::<Vec<usize>>();

        let ids: Vec<&RecordID> = records.keys().collect();
        for (i, cluster_id) in assignments.iter().enumerate() {
            // The number of records in the cluster.
            let count = self.clusters[*cluster_id].len() as f32;
            let new_count = count + 1.0;

            // This updates the centroid of the cluster by taking the
            // weighted average of the existing centroid and the new
            // vector that is being inserted.
            let centroid: Vec<f32> = self.centroids[*cluster_id]
                .to_vec()
                .par_iter()
                .zip(vectors[i].to_vec().par_iter())
                .map(|(c, v)| ((c * count) + v) / new_count)
                .collect();

            self.centroids[*cluster_id] = centroid.into();
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

    /// Creates the codebook for the Product Quantization.
    /// - `vectors`: Dataset to create the codebook from.
    ///
    /// The size of the dataset should be large enough to cover the
    /// entire vector space to ensure the codebook represents the
    /// distribution of the vectors accurately.
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
                .map(|centroid| centroid.to_owned())
                .collect();
        }
    }

    /// Finds the nearest centroids to a vector for cluster assignments.
    /// - `vector`: Full-length vector.
    /// - `k`: Number of centroids to find.
    fn find_nearest_centroids(&self, vector: &Vector, k: usize) -> Vec<usize> {
        let mut distances: Vec<(usize, f32)> = self
            .centroids
            .par_iter()
            .enumerate()
            .map(|(i, center)| (i, self.metric().distance(center, vector)))
            .collect();

        distances.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        distances.into_iter().take(k).map(|(i, _)| i).collect()
    }

    /// Finds the nearest centroid in the codebook for a subvector.
    /// - `part_index`: Quantization part index.
    /// - `subvector`: Subvector to quantize.
    ///
    /// Part index is used to determine which part of the vector to
    /// quantize. For example, if we have a vector with 4 dimensions and
    /// we want to quantize it into two parts:
    ///
    /// ```text
    /// [1, 2, 3, 4] => [[1, 2], [3, 4]]
    /// part_index   =>    0       1
    /// ```
    fn find_nearest_code(
        &self,
        part_index: usize,
        subvector: &Vector,
    ) -> usize {
        self.codebook[part_index]
            .par_iter()
            .map(|centroid| self.metric().distance(centroid, subvector))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap_or_default()
            .0
    }

    /// Quantizes a full-length vector into a PQ vector.
    /// - `vector`: Vector data.
    fn quantize_vector(&self, vector: &Vector) -> VectorPQ {
        (0..self.params.sub_dimension as usize)
            .into_par_iter()
            .map(|i| {
                let subvector = self.get_subvector(i, vector);
                self.find_nearest_code(i, &subvector) as u8
            })
            .collect::<Vec<u8>>()
            .into()
    }

    /// Reconstructs a full-length vector from a PQ vector.
    /// - `vector_pq`: PQ vector data.
    fn dequantize_vector(&self, vector_pq: &VectorPQ) -> Vector {
        vector_pq
            .0
            .par_iter()
            .enumerate()
            .map(|(i, code_id)| self.codebook[i][*code_id as usize].to_vec())
            .flatten()
            .collect::<Vec<f32>>()
            .into()
    }

    /// Extracts a subvector from a full-length vector.
    /// - `part_index`: Quantization part index.
    /// - `vector`: Full-length vector.
    fn get_subvector(&self, part_index: usize, vector: &Vector) -> Vector {
        let dim = vector.len() / self.params.sub_dimension as usize;
        let start = part_index * dim;
        let end = (part_index + 1) * dim;
        let subvector = vector.0[start..end].to_vec();
        Vector(subvector.into_boxed_slice())
    }
}

impl IndexOps for IndexIVFPQ {
    fn new(params: impl IndexParams) -> Result<Self, Error> {
        let params = downcast_params::<ParamsIVFPQ>(params)?;
        let codebook = vec![vec![]; params.sub_dimension as usize];
        let clusters = vec![vec![]; params.centroids];

        let index = IndexIVFPQ {
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
                // Skip hidden records.
                if self.metadata.hidden.contains(&record_id) {
                    continue;
                }

                let record = self.data.get(&record_id).unwrap();
                let data = record.data.clone();

                // Skip records that don't pass the filters.
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

/// Parameters for IndexIVFPQ.
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

        let mut index = IndexIVFPQ::new(params).unwrap();
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

        let mut index = IndexIVFPQ::new(params).unwrap();
        index_tests::populate_index(&mut index);
        index_tests::test_basic_search(&index);
        index_tests::test_advanced_search(&index);
    }
}
