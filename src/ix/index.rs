use super::*;

#[derive(Clone, Copy)]
pub struct Node<M: Copy, const N: usize> {
    pub key: &'static str,
    pub vector: Vector<N>,
    pub metadata: M,
}

#[derive(Debug)]
pub struct QueryResult<M: Copy> {
    pub key: &'static str,
    pub distance: f32,
    pub metadata: M,
}

pub struct Index<M: Copy, const N: usize> {
    trees: Vec<Tree<N>>,
    metadata: HashMap<&'static str, M>,
    vectors: HashMap<&'static str, Vector<N>>,
    config: IndexConfig,
}

#[derive(Clone, Copy)]
pub struct IndexConfig {
    pub num_trees: i32,
    pub max_leaf_size: i32,
}

impl<M: Copy, const N: usize> Index<M, N> {
    fn deduplicate(nodes: &Vec<Node<M, N>>) -> Vec<Node<M, N>> {
        let mut unique_nodes = vec![];
        let hashes_seen = DashSet::new();

        for node in nodes {
            let hash_key = node.vector.to_hashkey();
            if !hashes_seen.contains(&hash_key) {
                hashes_seen.insert(hash_key);
                unique_nodes.push(*node);
            }
        }

        unique_nodes
    }

    pub fn new(config: &IndexConfig) -> Index<M, N> {
        Index::<M, N> {
            trees: vec![],
            metadata: HashMap::new(),
            vectors: HashMap::new(),
            config: *config,
        }
    }

    pub fn build(nodes: &Vec<Node<M, N>>, config: &IndexConfig) -> Index<M, N> {
        let nodes = Self::deduplicate(nodes);

        let keys = nodes.iter().map(|node| node.key).collect();

        let mut metadata = HashMap::new();
        let mut vectors = HashMap::new();

        for node in nodes.iter() {
            metadata.insert(node.key, node.metadata);
            vectors.insert(node.key, node.vector);
        }

        let trees: Vec<Tree<N>> = (0..config.num_trees)
            .map(|_| Tree::build(&keys, &vectors, config.max_leaf_size))
            .collect();

        let config = *config;

        Index::<M, N> { trees, metadata, vectors, config }
    }

    pub fn insert(&mut self, node: &Node<M, N>) {
        self.metadata.insert(node.key, node.metadata);
        self.vectors.insert(node.key, node.vector);

        if self.trees.is_empty() {
            for _ in 0..self.config.num_trees {
                self.trees.push(Tree::Leaf(Box::new(vec![node.key])));
            }
        }

        for tree in self.trees.iter_mut() {
            let data = (node.key, &node.vector);
            tree.insert(data, &self.vectors, self.config.max_leaf_size);
        }
    }

    pub fn delete(&mut self, key: &'static str) {
        for tree in self.trees.iter_mut() {
            let data = (key, &self.vectors[key]);
            tree.delete(data);
        }

        self.metadata.remove(key);
        self.vectors.remove(key);
    }

    pub fn query(&self, vector: &Vector<N>, n: i32) -> Vec<QueryResult<M>> {
        let candidates = DashSet::new();

        self.trees.iter().for_each(|tree| {
            tree.query(&candidates, vector, n);
        });

        let sorted_candidates: Vec<_> = candidates
            .into_iter()
            .map(|key| (key, self.vectors[key].euclidean_distance(vector)))
            .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .take(n as usize)
            .collect();

        let mut result = vec![];

        for (key, distance) in sorted_candidates.iter() {
            let metadata = self.metadata[key];
            result.push(QueryResult { key, distance: *distance, metadata });
        }

        result
    }
}
