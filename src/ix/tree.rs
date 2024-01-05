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
}

pub type Leaf<const N: usize> = Vec<&'static str>;

pub struct Branch<const N: usize> {
    pub hyperplane: Hyperplane<N>,
    pub left_tree: Tree<N>,
    pub right_tree: Tree<N>,
}
