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
    vectors: &'a [Vector<N>],
    config: &'a IndexConfig,
}

impl<'a, const M: usize, const N: usize> IndexConstruction<'a, M, N> {
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

            if current_layer > *layer {
                let layer = layers[current_layer.0 - 1].as_slice();
                search.search(layer, vector, self.vectors, M);
                search.cull();
            } else {
                search.search(self.base_layer, vector, self.vectors, M);
                break;
            }
        }

        let candidates = {
            let candidates = search.select_simple();
            &candidates[..Ord::min(candidates.len(), M)]
        };

        for (i, candidate) in candidates.iter().enumerate() {
            let vec_id = candidate.vector_id;
            let old = &self.vectors[&vec_id];

            let distance = candidate.distance;
            let comparator = |id: &VectorID| {
                if !id.is_valid() {
                    Ordering::Greater
                } else {
                    let other = &self.vectors[id];
                    distance.cmp(&old.distance(other).into())
                }
            };

            let index = self.base_layer[&vec_id]
                .read()
                .binary_search_by(|id| comparator(&id))
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
pub struct IndexGraph<D, const N: usize, const M: usize> {
    pub data: Vec<D>,
    pub config: IndexConfig,
    vectors: Vec<Vector<N>>,
    base_layer: Vec<BaseNode<M>>,
    upper_layers: Vec<Vec<UpperNode<M>>>,
}

impl<D, const N: usize, const M: usize> Index<&VectorID>
    for IndexGraph<D, N, M>
{
    type Output = Vector<N>;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self.vectors[index.0 as usize]
    }
}

impl<D: Copy, const N: usize, const M: usize> IndexGraph<D, N, M> {
    /// Creates an empty index graph.
    /// * `config`: The index configuration.
    pub fn new(config: &IndexConfig) -> Self {
        Self {
            config: *config,
            data: vec![],
            vectors: vec![],
            base_layer: vec![],
            upper_layers: vec![],
        }
    }

    /// Builds an index graph from a list of vectors.
    /// * `config`: The index configuration.
    /// * `data`: Data associated with the vectors.
    /// * `vectors`: The vectors to index.
    pub fn build(
        config: &IndexConfig,
        data: &Vec<D>,
        vectors: &Vec<Vector<N>>,
    ) -> Self {
        let mut rng = SmallRng::seed_from_u64(config.seed);

        if vectors.is_empty() {
            return Self::new(config);
        }

        let mut len = vectors.len();
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
        assert!(vectors.len() < u32::MAX as usize);

        let mut shuffler = |i: usize| {
            let x = rng.gen_range(0..vectors.len() as usize);
            (VectorID(x as u32), i)
        };

        let mut shuffled = (0..vectors.len())
            .map(|i| shuffler(i))
            .collect::<Vec<(VectorID, usize)>>();

        shuffled.sort_unstable();

        let mut output = vec![INVALID; vectors.len()];
        let mut output_mapper = |(i, index)| {
            output[index] = VectorID(i as u32);
            vectors[index as usize].clone()
        };

        let vectors = shuffled
            .into_iter()
            .enumerate()
            .map(|(i, item)| output_mapper((i, item.1)))
            .collect::<Vec<Vector<N>>>();

        let mut ranges = Vec::with_capacity(top_layer.0);
        for (i, (size, cumulative)) in layers.into_iter().enumerate() {
            let start = cumulative - size;
            let layer_id = LayerID(num_layers - i - 1);
            let value = max(start, 1)..cumulative;
            ranges.push((layer_id, value));
        }

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

            if !layer.is_zero() {
                (&state.base_layer[..end])
                    .into_par_iter()
                    .map(|zero| UpperNode::from_zero(&zero.read()))
                    .collect_into_vec(&mut upper_layers[layer.0 - 1]);
            }
        }

        let mut sorted = output.par_iter().enumerate().collect::<Vec<_>>();
        sorted.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        let data = sorted.iter().map(|item| data[item.0]).collect();

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

        search.visited.reserve_capacity(self.vectors.len());
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
            let distance = candidate.distance.0;
            let data = self.data[candidate.vector_id.0 as usize].clone();
            SearchResult { distance, data }
        };

        search.iter().map(|candidate| map_result(candidate)).take(n).collect()
    }
}

/// The index graph search result.
/// * `D`: Data associated with the vector.
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult<D> {
    pub distance: f32,
    pub data: D,
}
