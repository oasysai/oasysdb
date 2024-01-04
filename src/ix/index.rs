use super::*;
use dashmap::DashSet;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use std::cmp::min;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Node<M: Copy, const N: usize> {
    pub key: &'static str,
    pub vector: Vector<N>,
    pub metadata: M,
}

#[derive(Debug, PartialEq)]
pub struct QueryResult<M: Copy> {
    pub key: &'static str,
    pub distance: f32,
    pub metadata: M,
}

pub struct Index<M: Copy, const N: usize> {
    trees: Vec<Tree<N>>,
    metadata: HashMap<&'static str, M>,
    vectors: HashMap<&'static str, Vector<N>>,
}

impl<M: Copy, const N: usize> Index<M, N> {
    fn build_hyperplane(
        keys: &Vec<&'static str>,
        vectors: &HashMap<&str, Vector<N>>,
    ) -> (Hyperplane<N>, Vec<&'static str>, Vec<&'static str>) {
        let mut rng = rand::thread_rng();
        let sample: Vec<_> = keys.choose_multiple(&mut rng, 2).collect();
        let (a, b) = (*sample[0], *sample[1]);

        let coefficients = vectors[a].subtract_from(&vectors[b]);
        let point_on_plane = vectors[a].average(&vectors[b]);
        let constant = -coefficients.dot_product(&point_on_plane);
        let hyperplane = Hyperplane::<N> { coefficients, constant };

        let mut left = vec![];
        let mut right = vec![];

        for key in keys.iter() {
            if hyperplane.point_is_above(&vectors[key]) {
                right.push(*key)
            } else {
                left.push(*key)
            };
        }

        (hyperplane, right, left)
    }

    fn build_tree(
        keys: &Vec<&'static str>,
        vectors: &HashMap<&str, Vector<N>>,
        max_size: i32,
    ) -> Tree<N> {
        if keys.len() <= max_size as usize {
            return Tree::Leaf(Box::new(keys.clone()));
        }

        let (plane, right, left) = Self::build_hyperplane(keys, vectors);
        let right_tree = Self::build_tree(&right, vectors, max_size);
        let left_tree = Self::build_tree(&left, vectors, max_size);

        Tree::Branch(Box::new(Branch::<N> {
            hyperplane: plane,
            left_tree,
            right_tree,
        }))
    }

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

    pub fn build(
        nodes: &Vec<Node<M, N>>,
        num_trees: i32,
        max_size: i32,
    ) -> Index<M, N> {
        let nodes = Self::deduplicate(nodes);

        let keys = nodes.iter().map(|node| node.key).collect();

        let mut metadata = HashMap::new();
        let mut vectors = HashMap::new();

        for node in nodes.iter() {
            metadata.insert(node.key, node.metadata);
            vectors.insert(node.key, node.vector);
        }

        let trees: Vec<_> = (0..num_trees)
            .into_iter()
            .map(|_| Self::build_tree(&keys, &vectors, max_size))
            .collect();

        return Index::<M, N> { trees, metadata, vectors };
    }

    fn candidates_from_leaf(
        candidates: &DashSet<&str>,
        leaf: &Box<Vec<&'static str>>,
        n: i32,
    ) -> i32 {
        let num_candidates = min(n as usize, leaf.len());
        for i in 0..num_candidates {
            candidates.insert(leaf[i]);
        }
        num_candidates as i32
    }

    fn candidates_from_branch(
        candidates: &DashSet<&str>,
        branch: &Box<Branch<N>>,
        vector: Vector<N>,
        n: i32,
    ) -> i32 {
        let above = (*branch).hyperplane.point_is_above(&vector);

        let (main_tree, backup_tree) = match above {
            true => (&(branch.right_tree), &(branch.left_tree)),
            false => (&(branch.left_tree), &(branch.right_tree)),
        };

        let num_candidates =
            Self::get_candidates(candidates, main_tree, vector, n);

        if num_candidates >= n {
            return num_candidates;
        }

        num_candidates
            + Self::get_candidates(
                candidates,
                backup_tree,
                vector,
                n - num_candidates,
            )
    }

    fn get_candidates(
        candidates: &DashSet<&str>,
        tree: &Tree<N>,
        vector: Vector<N>,
        n: i32,
    ) -> i32 {
        match tree {
            Tree::Leaf(leaf) => Self::candidates_from_leaf(candidates, leaf, n),
            Tree::Branch(branch) => {
                Self::candidates_from_branch(candidates, branch, vector, n)
            }
        }
    }

    pub fn query(&self, vector: Vector<N>, n: i32) -> Vec<QueryResult<M>> {
        let candidates = DashSet::new();

        self.trees.iter().for_each(|tree| {
            Self::get_candidates(&candidates, tree, vector, n);
        });

        let sorted_candidates: Vec<_> = candidates
            .into_iter()
            .map(|key| (key, self.vectors[key].euclidean_distance(&vector)))
            .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .take(n as usize)
            .collect();

        let mut result = vec![];

        for (key, distance) in sorted_candidates.iter() {
            let metadata = self.metadata[key];
            result.push(QueryResult {
                key: *key,
                distance: *distance,
                metadata,
            });
        }

        result
    }
}
