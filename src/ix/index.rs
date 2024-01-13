use super::*;

/// The HNSW index graph configuration.
/// * `ef_construction`: Nodes to consider during construction.
/// * `ef_search`: Nodes to consider during search.
/// * `ml`: Layer multiplier. The optimal value is `1/ln(M)`.
/// * `seed`: Seed for random number generator.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct IndexConfig {
    pub ef_construction: usize,
    pub ef_search: usize,
    pub ml: f32,
    pub seed: u64,
}

impl Default for IndexConfig {
    /// Default configuration for the HNSW index graph.
    /// * `ef_construction`: 40
    /// * `ef_search`: 15
    /// * `ml`: 0.3
    /// * `seed`: Randomized integer
    fn default() -> Self {
        let ml = 0.3;
        let seed: u64 = random();
        Self { ef_construction: 40, ef_search: 15, ml, seed }
    }
}

struct IndexConstruction<'a, const M: usize, const N: usize> {
    search_pool: SearchPool<M, N>,
    top_layer: LayerID,
    base_layer: &'a [RwLock<BaseNode<M>>],
    vectors: &'a HashMap<VectorID, Vector<N>>,
    config: &'a IndexConfig,
}

impl<'a, const M: usize, const N: usize> IndexConstruction<'a, M, N> {
    /// Inserts a vector ID into a layer.
    /// * `vector_id`: The vector ID to insert.
    /// * `layer`: The layer to insert into.
    /// * `layers`: The upper layers.
    fn insert(
        &self,
        vector_id: &VectorID,
        layer: &LayerID,
        layers: &[Vec<UpperNode<M>>],
    ) {
        let vector = &self.vectors[vector_id];
        let mut node = self.base_layer[vector_id].write();

        let (mut search, mut insertion) = self.search_pool.pop();
        insertion.ef = self.config.ef_construction;

        search.reset();
        search.push(&VectorID(0), vector, self.vectors);

        for current_layer in self.top_layer.descend() {
            if current_layer <= *layer {
                search.ef = self.config.ef_construction;
            }

            // Find the nearest neighbor candidates.
            if current_layer > *layer {
                let layer = layers[current_layer.0 - 1].as_slice();
                search.search(layer, vector, self.vectors, M);
                search.cull();
            } else {
                search.search(self.base_layer, vector, self.vectors, M);
                break;
            }
        }

        // Select the neighbors.
        let candidates = {
            let candidates = search.select_simple();
            &candidates[..Ord::min(candidates.len(), M)]
        };

        for (i, candidate) in candidates.iter().enumerate() {
            let vec_id = candidate.vector_id;
            let old = &self.vectors[&vec_id];
            let distance = candidate.distance;

            // Function to sort the vectors by distance.
            let ordering = |id: &VectorID| {
                if !id.is_valid() {
                    Ordering::Greater
                } else {
                    let other = &self.vectors[id];
                    distance.cmp(&old.distance(other).into())
                }
            };

            // Find the correct index to insert at to keep the order.
            let index = self.base_layer[&vec_id]
                .read()
                .binary_search_by(|id| ordering(&id))
                .unwrap_or_else(|error| error);

            self.base_layer[&vec_id].write().insert(index, vector_id);
            node.set(i, vector_id);
        }

        self.search_pool.push(&(search, insertion));
    }
}

/// The HNSW index graph.
/// * `D`: Data associated with the vector.
/// * `N`: The vector dimension.
/// * `M`: Maximum neighbors per vector node.
#[derive(Serialize, Deserialize)]
pub struct IndexGraph<D, const N: usize, const M: usize = 32> {
    pub config: IndexConfig,
    pub data: HashMap<VectorID, D>,
    vectors: HashMap<VectorID, Vector<N>>,
    base_layer: Vec<BaseNode<M>>,
    upper_layers: Vec<Vec<UpperNode<M>>>,
}

impl<D, const N: usize, const M: usize> Index<&VectorID>
    for IndexGraph<D, N, M>
{
    type Output = Vector<N>;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self.vectors[index]
    }
}

impl<D: Copy, const N: usize, const M: usize> IndexGraph<D, N, M> {
    /// Creates an empty index graph.
    /// * `config`: The index configuration.
    pub fn new(config: &IndexConfig) -> Self {
        Self {
            config: *config,
            data: HashMap::new(),
            vectors: HashMap::new(),
            base_layer: vec![],
            upper_layers: vec![],
        }
    }

    /// Builds an index graph from a list of vectors.
    /// * `config`: The index configuration.
    /// * `data`: Data associated with the vectors.
    /// * `vectors`: The vectors to index.
    pub fn build(config: &IndexConfig, records: &[IndexRecord<D, N>]) -> Self {
        if records.is_empty() {
            return Self::new(config);
        }

        // Find the number of layers.

        let mut len = records.len();
        let mut layers = Vec::new();

        loop {
            let next = (len as f32 * config.ml) as usize;

            if next < M {
                break;
            }

            layers.push((len - next, len));
            len = next;
        }

        layers.push((len, len));
        layers.reverse();

        let num_layers = layers.len();
        let top_layer = LayerID(num_layers - 1);

        // Ensure the number of vectors is less than u32 capacity.
        assert!(records.len() < u32::MAX as usize);

        // Give all vectors a random layer and sort the list of nodes
        // by descending order for construction.

        // This allows us to copy higher layers to lower layers as
        // construction progresses, while preserving randomness in
        // each point's layer and insertion order.

        let vectors = records
            .into_iter()
            .enumerate()
            .map(|(i, item)| (VectorID(i as u32), item.vector))
            .collect::<HashMap<VectorID, Vector<N>>>();

        // Figure out how many nodes will go on each layer.
        // This helps us allocate memory capacity for each
        // layer in advance, and also helps enable batch
        // insertion of points.

        let mut ranges = Vec::with_capacity(top_layer.0);
        for (i, (size, cumulative)) in layers.into_iter().enumerate() {
            let start = cumulative - size;
            let layer_id = LayerID(num_layers - i - 1);
            let value = max(start, 1)..cumulative;
            ranges.push((layer_id, value));
        }

        // Initialize data for layers.

        let search_pool = SearchPool::new(vectors.len());
        let mut upper_layers = vec![vec![]; top_layer.0];
        let base_layer = vectors
            .par_iter()
            .map(|_| RwLock::new(BaseNode::default()))
            .collect::<Vec<_>>();

        let state = IndexConstruction {
            base_layer: &base_layer,
            search_pool,
            top_layer,
            vectors: &vectors,
            config: &config,
        };

        for (layer, range) in ranges {
            let inserter = |id| state.insert(&id, &layer, &upper_layers);
            let end = range.end;

            if layer == top_layer {
                range.into_iter().for_each(|i| inserter(VectorID(i as u32)))
            } else {
                range.into_par_iter().for_each(|i| inserter(VectorID(i as u32)))
            }

            // Copy the base layer state to the upper layer.
            if !layer.is_zero() {
                (&state.base_layer[..end])
                    .into_par_iter()
                    .map(|zero| UpperNode::from_zero(&zero.read()))
                    .collect_into_vec(&mut upper_layers[layer.0 - 1]);
            }
        }

        let data = records
            .into_iter()
            .enumerate()
            .map(|(i, item)| (VectorID(i as u32), item.data))
            .collect::<HashMap<VectorID, D>>();

        // Unwrap the base nodes for the base layer.
        let base_iter = base_layer.into_iter();
        let base_layer = base_iter.map(|node| node.into_inner()).collect();

        Self { data, vectors, base_layer, upper_layers, config: *config }
    }

    /// Searches the index graph for the nearest neighbors.
    /// * `vector`: The vector to search.
    /// * `n`: Number of neighbors to return.
    pub fn search<'a>(
        &'a self,
        vector: &'a Vector<N>,
        n: usize,
    ) -> Vec<SearchResult<D>> {
        let mut search: Search<M, N> = Search::default();

        if self.vectors.is_empty() {
            return vec![];
        }

        search.visited.resize_capacity(self.vectors.len());
        search.push(&VectorID(0), vector, &self.vectors);

        for layer in LayerID(self.upper_layers.len()).descend() {
            search.ef = if layer.is_zero() { self.config.ef_search } else { 5 };

            if layer.0 == 0 {
                let layer = self.base_layer.as_slice();
                search.search(layer, vector, &self.vectors, M);
            } else {
                let layer = self.upper_layers[layer.0 - 1].as_slice();
                search.search(layer, vector, &self.vectors, M);
            }

            if !layer.is_zero() {
                search.cull();
            }
        }

        let map_result = |candidate: Candidate| {
            let id = candidate.vector_id.0;
            let distance = candidate.distance.0;
            let data = *self.data.get(&candidate.vector_id).unwrap();
            SearchResult { id, distance, data }
        };

        search.iter().map(|candidate| map_result(candidate)).take(n).collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IndexRecord<D, const N: usize> {
    pub vector: Vector<N>,
    pub data: D,
}

/// The index graph search result.
/// * `D`: Data associated with the vector.
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult<D> {
    pub id: u32, // Vector ID
    pub distance: f32,
    pub data: D,
}
