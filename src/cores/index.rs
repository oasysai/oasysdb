use super::*;
use crate::utils::kmeans::KMeans;
use std::rc::Rc;

type ClusterIndex = usize;

/// ANNS Index interface.
///
/// OasysDB uses a modified version of IVF index algorithm. This custom index
/// implementation allows OasysDB to maintain a balanced index structure
/// allowing the clusters to grow to accommodate data growth.
#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    centroids: Vec<Vector>,
    clusters: Vec<Vec<RecordID>>,

    // Index parameters.
    metric: Metric,
    density: usize,
}

impl Index {
    /// Create a new index instance with default parameters.
    ///
    /// Default parameters:
    /// - metric: Euclidean
    /// - density: 256
    pub fn new() -> Self {
        Index {
            centroids: vec![],
            clusters: vec![],
            metric: Metric::Euclidean,
            density: 256,
        }
    }

    /// Configure the metric used for distance calculations.
    pub fn with_metric(mut self, metric: Metric) -> Self {
        self.metric = metric;
        self
    }

    /// Configure the density of the index.
    pub fn with_density(mut self, density: usize) -> Self {
        self.density = density;
        self
    }

    /// Insert a new record into the index.
    ///
    /// This method required the reference to all the records because
    /// during the cluster splitting process, the record assignments
    /// will be re-calculated
    pub fn insert(
        &mut self,
        id: &RecordID,
        record: &Record,
        records: &HashMap<RecordID, Record>,
    ) -> Result<(), Status> {
        let vector = &record.vector;
        let nearest_centroid = self.find_nearest_centroid(vector);

        // If the index is empty, the record's vector will be
        // the first centroid.
        if nearest_centroid.is_none() {
            let cluster_id = self.insert_centroid(vector);
            self.clusters[cluster_id].push(*id);
            return Ok(());
        }

        let nearest_centroid = nearest_centroid.unwrap();
        if self.clusters[nearest_centroid].len() < self.density {
            self.update_centroid(&nearest_centroid, vector);
            self.clusters[nearest_centroid].push(*id);
        } else {
            // If the cluster is full, insert the record into the cluster
            // and split the cluster with KMeans algorithm.
            self.clusters[nearest_centroid].push(*id);
            self.split_cluster(&nearest_centroid, records);
        }

        Ok(())
    }

    /// Delete a record from the index by its ID.
    ///
    /// This method will iterate over all the clusters and remove the record
    /// from the cluster if it exists. This method doesn't update the value of
    /// the cluster's centroid.
    pub fn delete(&mut self, id: &RecordID) -> Result<(), Status> {
        // Find the cluster and record indices where the record is stored.
        let cluster_record_index =
            self.clusters.iter().enumerate().find_map(|(i, cluster)| {
                cluster.par_iter().position_first(|x| x == id).map(|x| (i, x))
            });

        if let Some((cluster_ix, record_ix)) = cluster_record_index {
            // If the cluster has only one record, remove the cluster and
            // centroid from the index. This won't happen often.
            if self.clusters[cluster_ix].len() == 1 {
                self.clusters.remove(cluster_ix);
                self.centroids.remove(cluster_ix);
            } else {
                self.clusters[cluster_ix].remove(record_ix);
            }
        }

        Ok(())
    }

    /// Insert a new centroid and cluster into the index.
    /// - vector: Centroid vector.
    fn insert_centroid(&mut self, vector: &Vector) -> ClusterIndex {
        self.centroids.push(vector.to_owned());
        self.clusters.push(vec![]);
        self.centroids.len() - 1
    }

    /// Recalculate the centroid of a cluster with the new vector.
    ///
    /// This method must be called before inserting the new vector into the
    /// cluster because this method calculates the new centroid by taking the
    /// weighted average of the current centroid and adding the new vector
    /// before normalizing the result with the new cluster size.
    fn update_centroid(&mut self, cluster_id: &ClusterIndex, vector: &Vector) {
        let count = self.clusters[*cluster_id].len() as f32;
        self.centroids[*cluster_id] = self.centroids[*cluster_id]
            .as_slice()
            .iter()
            .zip(vector.as_slice())
            .map(|(a, b)| (a * count) + b / count + 1.0)
            .collect::<Vec<f32>>()
            .into();
    }

    /// Find the nearest centroid to a given vector.
    ///
    /// If the index is empty, this method will return None. Otherwise, it will
    /// calculate the distance between the given vector and all centroids and
    /// return the index of the centroid with the smallest distance.
    fn find_nearest_centroid(&self, vector: &Vector) -> Option<ClusterIndex> {
        self.centroids
            .par_iter()
            .map(|centroid| self.metric.distance(centroid, vector))
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(index, _)| index)
    }

    /// Split a cluster into two new clusters.
    ///
    /// The current cluster will be halved. The first half will be assigned to
    /// the current cluster, and the second half will be assigned to a new
    /// cluster with a new centroid.
    fn split_cluster(
        &mut self,
        cluster_id: &ClusterIndex,
        records: &HashMap<RecordID, Record>,
    ) {
        let record_ids = &self.clusters[*cluster_id];
        let vectors = record_ids
            .iter()
            .map(|id| &records.get(id).unwrap().vector)
            .collect::<Vec<&Vector>>();

        let mut kmeans = KMeans::new(2).with_metric(self.metric);
        kmeans.fit(Rc::from(vectors)).unwrap();

        let centroids = kmeans.centroids();
        self.centroids[*cluster_id] = centroids[0].to_owned();
        self.centroids.push(centroids[1].to_owned());

        let mut clusters = [vec![], vec![]];
        let assignments = kmeans.assignments();
        for (i, cluster_id) in assignments.iter().enumerate() {
            clusters[*cluster_id].push(record_ids[i]);
        }

        self.clusters[*cluster_id] = clusters[0].to_vec();
        self.clusters.push(clusters[1].to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_many() {
        let params = Parameters::default();
        let mut index = setup_index(&params);

        let mut records = HashMap::new();
        for _ in 0..1000 {
            let id = RecordID::new();
            let record = Record::random(params.dimension);
            records.insert(id, record);
        }

        for (id, record) in records.iter() {
            index.insert(id, record, &records).unwrap();
        }

        assert!(index.centroids.len() > 20);
    }

    #[test]
    fn test_delete() {
        let params = Parameters::default();
        let mut index = setup_index(&params);

        let mut ids = vec![];
        for _ in 0..10 {
            let centroid = Vector::random(params.dimension);
            let mut cluster = vec![];
            for _ in 0..10 {
                let id = RecordID::new();
                cluster.push(id);
                ids.push(id);
            }

            index.centroids.push(centroid);
            index.clusters.push(cluster);
        }

        assert_eq!(ids.len(), 100);
        assert_eq!(index.centroids.len(), 10);

        index.delete(&ids[0]).unwrap();
        for cluster in index.clusters.iter() {
            assert!(!cluster.contains(&ids[0]));
        }

        for i in 1..10 {
            index.delete(&ids[i]).unwrap();
        }

        assert_eq!(index.centroids.len(), 9);
    }

    #[test]
    fn test_insert_centroid() {
        let params = Parameters::default();
        let mut index = setup_index(&params);

        let vector = Vector::random(params.dimension);
        let cluster_id = index.insert_centroid(&vector);

        assert_eq!(index.centroids.len(), 1);
        assert_eq!(index.clusters.len(), 1);

        assert_eq!(index.centroids[0], vector);
        assert_eq!(cluster_id, 0);
    }

    #[test]
    fn test_update_centroid() {
        let params = Parameters::default();
        let mut index = setup_index(&params);

        let initial_centroid = Vector::from(vec![0.0; params.dimension]);
        let cluster_id = index.insert_centroid(&initial_centroid);
        index.clusters[cluster_id].push(RecordID::new());

        let vector = Vector::from(vec![1.0; params.dimension]);
        index.update_centroid(&cluster_id, &vector);

        let centroid = Vector::from(vec![0.5; params.dimension]);
        assert_ne!(index.centroids[cluster_id], centroid);
    }

    #[test]
    fn test_find_nearest_centroid_empty() {
        let params = Parameters::default();
        let index = setup_index(&params);

        let query = Vector::random(params.dimension);
        assert_eq!(index.find_nearest_centroid(&query), None);
    }

    #[test]
    fn test_find_nearest_centroid() {
        let params = Parameters::default();
        let mut index = setup_index(&params);

        for i in 1..5 {
            let centroid = Vector::from(vec![i as f32; params.dimension]);
            index.centroids.push(centroid);
        }

        let query = Vector::from(vec![0.0; params.dimension]);
        assert_eq!(index.find_nearest_centroid(&query), Some(0));
    }

    #[test]
    fn test_split_cluster() {
        let params = Parameters::default();
        let mut index = setup_index(&params);

        let mut ids = vec![];
        let mut records = HashMap::new();
        for i in 1..5 {
            let id = RecordID::new();
            let vector = Vector::from(vec![i as f32; params.dimension]);
            let record = Record { vector, metadata: HashMap::new() };

            ids.push(id);
            records.insert(id, record);
        }

        let centroid = Vector::from(vec![2.5; params.dimension]);
        index.centroids.push(centroid);
        index.clusters.push(ids);

        index.split_cluster(&0, &records);
        assert_eq!(index.centroids.len(), 2);
    }

    fn setup_index(params: &Parameters) -> Index {
        let index = Index::new()
            .with_metric(params.metric)
            .with_density(params.density);

        index
    }
}
