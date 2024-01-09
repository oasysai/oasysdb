use crate::ix::index::*;
use crate::ix::vector::*;
use rand::random;

mod test_database;
mod test_index;

fn create_test_index() -> Index<&'static str, 128> {
    let nodes = generate_nodes(100);
    let config = IndexConfig::default();
    Index::build(&nodes, &config)
}

fn generate_nodes(len: usize) -> Vec<Node<&'static str, 128>> {
    let mut nodes = vec![];
    for _ in 0..len {
        nodes.push(generate_node());
    }

    nodes
}

fn generate_node() -> Node<&'static str, 128> {
    let mut vector: Vector<128> = [0.0; 128];
    for float in &mut vector {
        *float = random::<f32>();
    }

    // Generate a random string key.
    let key = Box::leak(Box::new(random::<u32>().to_string()));
    Node { key, vector, metadata: key }
}
