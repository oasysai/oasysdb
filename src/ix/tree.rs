use super::*;

/// A tree that can be used to index vectors.
pub enum Tree<const N: usize> {
    Branch(Box<Branch<N>>),
    Leaf(Box<Leaf<N>>),
}

impl<const N: usize> Tree<N> {
    /// Builds a new index tree from scratch.
    /// * `keys` - The keys of vectors.
    /// * `vectors` - Mapping of keys to vectors.
    /// * `max_leaf_size` - The maximum number of keys in a leaf.
    pub fn build(
        keys: &Vec<&'static str>,
        vectors: &HashMap<&str, Vector<N>>,
        max_leaf_size: i32,
    ) -> Tree<N> {
        // Put all keys in a leaf if they fit.
        if keys.len() <= max_leaf_size as usize {
            return Tree::Leaf(Box::new(keys.clone()));
        }

        // Build a hyperplane to divide the vectors.
        let (plane, right, left) = Hyperplane::build(keys, vectors);

        // For each vector group, build a tree recursively.
        let right_tree = Self::build(&right, vectors, max_leaf_size);
        let left_tree = Self::build(&left, vectors, max_leaf_size);

        Tree::Branch(Box::new(Branch::<N> {
            hyperplane: plane,
            left_tree,
            right_tree,
        }))
    }

    /// Inserts a key into the tree with the given vector.
    /// * `data` - The new key and vector to insert.
    /// * `vectors` - Mapping of keys to vectors.
    /// * `max_leaf_size` - The maximum number of keys in a leaf.
    pub fn insert(
        &mut self,
        data: (&'static str, &Vector<N>),
        vectors: &HashMap<&'static str, Vector<N>>,
        max_leaf_size: i32,
    ) {
        if let Tree::Leaf(leaf) = self {
            leaf.push(data.0);

            // If the leaf is too big, rebuild it.
            if leaf.len() > max_leaf_size as usize {
                *self = Self::build(leaf, vectors, max_leaf_size);
            }
        } else if let Tree::Branch(branch) = self {
            match branch.hyperplane.point_is_above(&data.1) {
                true => &mut branch.right_tree,
                false => &mut branch.left_tree,
            }
            .insert(data, vectors, max_leaf_size);
        }
    }

    /// Deletes a key from the tree.
    /// * `data` - The key and vector to delete.
    pub fn delete(&mut self, data: (&'static str, &Vector<N>)) {
        if let Tree::Leaf(leaf) = self {
            leaf.retain(|&item| item != data.0);
        } else if let Tree::Branch(branch) = self {
            match branch.hyperplane.point_is_above(&data.1) {
                true => &mut branch.right_tree,
                false => &mut branch.left_tree,
            }
            .delete(data);
        }
    }

    /// Queries the tree for the nearest neighbors of a vector.
    /// * `candidates` - The set of candidates to add to.
    /// * `vector` - The vector to query.
    /// * `n` - The number of candidates to find.
    pub fn query(
        &self,
        candidates: &DashSet<&str>,
        vector: &Vector<N>,
        n: i32,
    ) -> i32 {
        if let Tree::Leaf(leaf) = self {
            // Collect the candidates from the leaf.
            let num_candidates = min(n as usize, leaf.len());
            for item in leaf.iter().take(num_candidates) {
                candidates.insert(item);
            }

            return num_candidates as i32;
        }

        // If we're not at a leaf, we're at a branch.
        let branch = match self {
            Tree::Branch(branch) => branch,
            _ => unreachable!(),
        };

        // Get the trees to query.
        let above = branch.hyperplane.point_is_above(vector);
        let (main_tree, backup_tree) = match above {
            true => (&branch.right_tree, &branch.left_tree),
            false => (&branch.left_tree, &branch.right_tree),
        };

        // Query the main tree for candidates.
        let main_candidates = main_tree.query(candidates, vector, n);

        // If we have enough candidates, return.
        if main_candidates >= n {
            return main_candidates;
        }

        // If we don't have enough candidates, query the backup tree.
        let n = n - main_candidates;
        let backup_candidates = backup_tree.query(candidates, vector, n);
        main_candidates + backup_candidates
    }
}

/// The edge of the tree that stores the vector keys.
pub type Leaf<const N: usize> = Vec<&'static str>;

/// A node that divides a certain vector space into two parts.
pub struct Branch<const N: usize> {
    pub hyperplane: Hyperplane<N>,
    pub left_tree: Tree<N>,
    pub right_tree: Tree<N>,
}
