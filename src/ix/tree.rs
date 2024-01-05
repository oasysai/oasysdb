use super::*;

pub enum Tree<const N: usize> {
    Branch(Box<Branch<N>>),
    Leaf(Box<Leaf<N>>),
}

impl<const N: usize> Tree<N> {
    pub fn build(
        keys: &Vec<&'static str>,
        vectors: &HashMap<&str, Vector<N>>,
        max_leaf_size: i32,
    ) -> Tree<N> {
        if keys.len() <= max_leaf_size as usize {
            return Tree::Leaf(Box::new(keys.clone()));
        }

        let (plane, right, left) = Hyperplane::build(keys, vectors);
        let right_tree = Self::build(&right, vectors, max_leaf_size);
        let left_tree = Self::build(&left, vectors, max_leaf_size);

        Tree::Branch(Box::new(Branch::<N> {
            hyperplane: plane,
            left_tree,
            right_tree,
        }))
    }

    fn candidates_from_leaf(
        &self,
        candidates: &DashSet<&str>,
        leaf: &Vec<&'static str>,
        n: i32,
    ) -> i32 {
        let num_candidates = min(n as usize, leaf.len());
        for item in leaf.iter().take(num_candidates) {
            candidates.insert(item);
        }
        num_candidates as i32
    }

    fn candidates_from_branch(
        &self,
        candidates: &DashSet<&str>,
        branch: &Branch<N>,
        vector: &Vector<N>,
        n: i32,
    ) -> i32 {
        let above = branch.hyperplane.point_is_above(vector);

        let (main_tree, backup_tree) = match above {
            true => (&branch.right_tree, &branch.left_tree),
            false => (&branch.left_tree, &branch.right_tree),
        };

        let num_candidates = main_tree.query(candidates, vector, n);

        if num_candidates >= n {
            return num_candidates;
        }

        num_candidates
            + backup_tree.query(candidates, vector, n - num_candidates)
    }

    pub fn query(
        &self,
        candidates: &DashSet<&str>,
        vector: &Vector<N>,
        n: i32,
    ) -> i32 {
        match self {
            Tree::Leaf(leaf) => self.candidates_from_leaf(candidates, leaf, n),
            Tree::Branch(branch) => {
                self.candidates_from_branch(candidates, branch, vector, n)
            }
        }
    }
}

pub type Leaf<const N: usize> = Vec<&'static str>;

pub struct Branch<const N: usize> {
    pub hyperplane: Hyperplane<N>,
    pub left_tree: Tree<N>,
    pub right_tree: Tree<N>,
}
