use super::*;

/// Input data structure to interact with the index.
/// * `M` - Any type of metadata for the vector.
/// * `N` - The vector dimension.
#[derive(Clone, Copy)]
pub struct Node<M: Copy, const N: usize> {
    pub key: &'static str,
    pub vector: Vector<N>,
    pub metadata: M,
}

/// Output data structure of the query operation.
#[derive(Debug)]
pub struct QueryResult<M: Copy> {
    pub key: &'static str,
    pub distance: f32,
    pub metadata: M,
}

/// Configuration for the vector index.
#[derive(Clone, Copy)]
pub struct IndexConfig {
    pub num_trees: i32,
    pub max_leaf_size: i32,
}

impl Default for IndexConfig {
    /// Default configuration for the vector index.
    /// * `num_trees`: 3
    /// * `max_leaf_size`: 15
    fn default() -> Self {
        IndexConfig { num_trees: 3, max_leaf_size: 15 }
    }
}

/// The vector index.
/// * `M` - Any type of metadata for the vector.
/// * `N` - The vector dimension.
pub struct Index<M: Copy, const N: usize> {
    // For memory efficiency, the trees only store the vector keys.
    // The vectors and the metadata are stored separately.
    trees: Vec<Tree<N>>,
    metadata: HashMap<&'static str, M>,
    vectors: HashMap<&'static str, Vector<N>>,
    config: IndexConfig,
}

impl<M: Copy, const N: usize> Index<M, N> {
    fn deduplicate(nodes: &Vec<Node<M, N>>) -> Vec<Node<M, N>> {
        let mut unique_nodes = vec![];
        let hashes_seen = DashSet::new();

        // Check if the hash key of the vector exists in the set.
        for node in nodes {
            let hash_key = node.vector.to_hashkey();
            if !hashes_seen.contains(&hash_key) {
                hashes_seen.insert(hash_key);
                unique_nodes.push(*node);
            }
        }

        unique_nodes
    }

    /// Returns the number of vector records in the index.
    pub fn count(&self) -> usize {
        self.vectors.len()
    }

    /// Lists vector nodes from the index.
    /// * `n` - The number of nodes to return.
    pub fn list(&self, n: usize) -> Vec<Node<M, N>> {
        let mut nodes = vec![];
        for (key, vector) in self.vectors.iter().take(n) {
            let metadata = self.metadata[key];
            nodes.push(Node { key, vector: *vector, metadata });
        }

        nodes
    }

    /// Creates a new blank index with the given configuration.
    /// * `config` - Configuration of the index.
    pub fn new(config: &IndexConfig) -> Index<M, N> {
        Index::<M, N> {
            trees: vec![],
            metadata: HashMap::new(),
            vectors: HashMap::new(),
            config: *config,
        }
    }

    /// Builds a new index from a list of nodes.
    /// * `nodes` - List of nodes.
    /// * `config` - Configuration of the index.
    pub fn build(nodes: &Vec<Node<M, N>>, config: &IndexConfig) -> Index<M, N> {
        // Remove duplicate node data.
        let nodes = Self::deduplicate(nodes);
        let keys = nodes.iter().map(|node| node.key).collect();

        // Create mapping metadata and vectors to store separately.
        let mut metadata = HashMap::new();
        let mut vectors = HashMap::new();
        for node in nodes.iter() {
            metadata.insert(node.key, node.metadata);
            vectors.insert(node.key, node.vector);
        }

        // Build the trees.
        let trees: Vec<Tree<N>> = (0..config.num_trees)
            .map(|_| Tree::build(&keys, &vectors, config.max_leaf_size))
            .collect();

        let config = *config;
        Index::<M, N> { trees, metadata, vectors, config }
    }

    /// Inserts a new node into the index.
    /// * `node` - The node to insert.
    pub fn insert(&mut self, node: &Node<M, N>) {
        self.metadata.insert(node.key, node.metadata);
        self.vectors.insert(node.key, node.vector);

        // If the trees are empty, create a new leaf.
        if self.trees.is_empty() {
            for _ in 0..self.config.num_trees {
                self.trees.push(Tree::Leaf(Box::new(vec![node.key])));
            }
        }

        // Insert the node into each tree.
        for tree in self.trees.iter_mut() {
            let data = (node.key, &node.vector);
            tree.insert(data, &self.vectors, self.config.max_leaf_size);
        }
    }

    /// Deletes a node from the index.
    /// * `key` - The key of the vector to delete.
    pub fn delete(&mut self, key: &'static str) {
        // Delete key from the trees.
        for tree in self.trees.iter_mut() {
            let data = (key, &self.vectors[key]);
            tree.delete(data);
        }

        // Delete related data.
        self.metadata.remove(key);
        self.vectors.remove(key);
    }

    /// Queries the index for the nearest neighbors of the given vector.
    /// * `vector` - The vector to query.
    /// * `n` - The number of candidates to find.
    pub fn query(&self, vector: &Vector<N>, n: i32) -> Vec<QueryResult<M>> {
        // Query each tree for nearest neighbors.
        let candidates = DashSet::new();
        self.trees.iter().for_each(|tree| {
            tree.query(&candidates, vector, n);
        });

        // Sort the candidates by distance.
        let sorted_candidates: Vec<_> = candidates
            .into_iter()
            .map(|key| (key, self.vectors[key].euclidean_distance(vector)))
            .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .take(n as usize)
            .collect();

        // Collect the results metadata.
        let mut result = vec![];
        for (key, distance) in sorted_candidates.iter() {
            let metadata = self.metadata[key];
            result.push(QueryResult { key, distance: *distance, metadata });
        }

        result
    }
}
