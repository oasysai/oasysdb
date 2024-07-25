use super::*;
use crate::utils::kmeans::{ClusterID, KMeans, Vectors};
use rand::seq::IteratorRandom;
use std::cmp::Ordering;
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
                    self.params.max_iterations,
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
    fn find_nearest_centroids(
        &self,
        vector: &Vector,
        k: usize,
    ) -> Vec<ClusterID> {
        let mut centroids = BinaryHeap::new();
        for (i, center) in self.centroids.iter().enumerate() {
            let id = ClusterID(i as u16);
            let distance = self.metric().distance(center, vector);

            let centroid = NearestCentroid { id, distance };
            centroids.push(centroid);

            if centroids.len() > k {
                centroids.pop();
            }
        }

        centroids
            .into_sorted_vec()
            .into_iter()
            .map(|centroid| centroid.id)
            .collect()
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

        // Validate the sampling parameter.
        if params.sampling <= 0.0 || params.sampling > 1.0 {
            let code = ErrorCode::RequestError;
            let message = "Sampling must be between 0.0 and 1.0.";
            return Err(Error::new(code, message));
        }

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

    fn build(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error> {
        let mut rng = rand::thread_rng();
        let sample = (records.len() as f32 * self.params.sampling) as usize;
        let vectors = records
            .values()
            .choose_multiple(&mut rng, sample)
            .par_iter()
            .map(|&record| &record.vector)
            .collect::<Vec<&Vector>>();

        // We use RC to avoid cloning the entire vector data as it
        // can be very large and expensive to clone.
        let vectors: Vectors = Rc::from(vectors.as_slice());
        self.create_codebook(vectors.clone());

        // Run KMeans to find the centroids for the IVF.
        let centroids = {
            let mut kmeans = KMeans::new(
                self.params.centroids,
                self.params.max_iterations,
                self.metric().to_owned(),
            );

            kmeans.fit(vectors.clone());
            kmeans.centroids().to_vec()
        };

        self.centroids = centroids;
        self.metadata.built = true;
        self.insert(records)?;
        Ok(())
    }

    fn insert(
        &mut self,
        records: HashMap<RecordID, Record>,
    ) -> Result<(), Error> {
        if records.is_empty() {
            return Ok(());
        }

        if !self.metadata().built {
            let code = ErrorCode::RequestError;
            let message = "Unable to insert records into an unbuilt index.";
            return Err(Error::new(code, message));
        }

        for (id, record) in records.iter() {
            let vector = &record.vector;
            let cid = self.find_nearest_centroids(vector, 1)[0].to_usize();

            // The number of records in the cluster.
            let count = self.clusters[cid].len() as f32;
            let new_count = count + 1.0;

            // This updates the centroid of the cluster by taking the
            // weighted average of the existing centroid and the new
            // vector that is being inserted.
            let centroid: Vec<f32> = self.centroids[cid]
                .to_vec()
                .par_iter()
                .zip(vector.to_vec().par_iter())
                .map(|(c, v)| ((c * count) + v) / new_count)
                .collect();

            self.centroids[cid] = centroid.into();
            self.clusters[cid].push(id.to_owned());
        }

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

    fn delete(&mut self, ids: Vec<RecordID>) -> Result<(), Error> {
        self.data.retain(|id, _| !ids.contains(id));
        self.clusters.par_iter_mut().for_each(|cluster| {
            cluster.retain(|id| !ids.contains(id));
        });

        Ok(())
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
            let cluster = &self.clusters[centroid_id.to_usize()];
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

    fn len(&self) -> usize {
        self.data.len()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Parameters for IndexIVFPQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsIVFPQ {
    /// Number of centroids or partitions in the IVF.
    pub centroids: usize,
    /// Maximum number of iterations to run the KMeans algorithm.
    pub max_iterations: usize,
    /// Number of centroids in the PQ sub-space.
    pub sub_centroids: u8,
    /// Dimension of the vector after PQ encoding.
    pub sub_dimension: u8,
    /// Number of clusters to explore during search.
    pub num_probes: u8,
    /// Fraction of the records for training the initial index.
    pub sampling: f32,
    /// Metric used to compute the distance between vectors.
    pub metric: DistanceMetric,
}

impl Default for ParamsIVFPQ {
    fn default() -> Self {
        Self {
            centroids: 256,
            max_iterations: 50,
            sub_centroids: 16,
            sub_dimension: 8,
            num_probes: 4,
            sampling: 0.1,
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

#[derive(Debug)]
struct NearestCentroid {
    id: ClusterID,
    distance: f32,
}

impl Eq for NearestCentroid {}

impl PartialEq for NearestCentroid {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for NearestCentroid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.partial_cmp(&other.distance).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for NearestCentroid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
            max_iterations: 10,
            sub_centroids: 8,
            sub_dimension: 2,
            sampling: 1.0,
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
            max_iterations: 20,
            sampling: 1.0,
            ..Default::default()
        };

        let mut index = IndexIVFPQ::new(params).unwrap();
        index_tests::populate_index(&mut index);
        index_tests::test_basic_search(&index);
        index_tests::test_advanced_search(&index);
    }
}
