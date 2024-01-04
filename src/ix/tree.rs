use super::*;

pub struct Hyperplane<const N: usize> {
    pub coefficients: Vector<N>,
    pub constant: f32,
}

impl<const N: usize> Hyperplane<N> {
    pub fn point_is_above(&self, point: &Vector<N>) -> bool {
        self.coefficients.dot_product(point) + self.constant >= 0.0
    }
}

pub enum Tree<const N: usize> {
    Branch(Box<Branch<N>>),
    Leaf(Box<Leaf<N>>),
}

pub type Leaf<const N: usize> = Vec<&'static str>;

pub struct Branch<const N: usize> {
    pub hyperplane: Hyperplane<N>,
    pub left_tree: Tree<N>,
    pub right_tree: Tree<N>,
}
