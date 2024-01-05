use oasysdb::*;
use rand::random;

fn main() {
    let n = 10;
    let nodes = generate_nodes(10000);
    let config = IndexConfig { num_trees: 5, max_leaf_size: 15 };
    let index = Index::build(&nodes, &config);

    let vector = generate_node().vector;
    let start = std::time::Instant::now();
    let index_result = index.query(&vector, n);
    println!("Index query duration: {:?}", start.elapsed().as_micros());

    let start = std::time::Instant::now();
    let exhaustive_result = search_exhaustive(&nodes, &vector, n);
    println!("Exhaustive query duration: {:?}", start.elapsed().as_micros());

    println!("Result Table:");
    println!("Index\t\t\t\t|Exhaustive");
    println!("Key \t\tDist \t\t|Key \t\tDist");
    for i in 0..n as usize {
        println!(
            "{}\t{:.4}\t\t|{}\t{:.4}",
            index_result[i].key,
            index_result[i].distance,
            exhaustive_result[i].0,
            exhaustive_result[i].1
        );
    }
}

fn search_exhaustive<M: Copy, const N: usize>(
    nodes: &[Node<M, N>],
    vector: &Vector<N>,
    n: i32,
) -> Vec<(&'static str, f32)> {
    let mut result: Vec<(&str, f32)> = nodes
        .iter()
        .map(|node| (node.key, node.vector.euclidean_distance(vector)))
        .collect();
    result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Return a set of IDs corresponding to the closest matches
    let mut final_candidates = vec![];

    for item in result.iter().take(n as usize) {
        final_candidates.push(*item);
    }

    final_candidates
}

fn generate_nodes(len: usize) -> Vec<Node<&'static str, 128>> {
    let mut nodes = vec![];

    for _ in 0..len {
        nodes.push(generate_node());
    }

    nodes
}

fn generate_node() -> Node<&'static str, 128> {
    let mut vector = [0.0; 128];

    for float in &mut vector {
        *float = random::<f32>();
    }

    let key: &'static str =
        Box::leak(random::<u32>().to_string().into_boxed_str());

    Node { key, vector, metadata: key }
}
