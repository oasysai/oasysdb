use oasysdb::*;
use rand::random;

fn main() {
    let len = 1000;
    let n = 10;
    let nodes = generate_nodes(len);
    let config = IndexConfig { num_trees: 3, max_leaf_size: 15 };
    let vector = generate_node().vector;

    let index_one_result = search_index_one(&nodes, &vector, &config, n);
    let index_two_result = search_index_two(&nodes, &vector, &config, n);
    let exhaustive_result = search_exhaustive(&nodes, &vector);

    for i in 0..n as usize {
        println!(
            "{:.4}\t{:.4}\t{:.4}",
            exhaustive_result[i].1,
            index_one_result[i].distance,
            index_two_result[i].distance,
        );
    }

    println!("...");
    println!("{:.4}", exhaustive_result[len - 1].1);
}

fn search_index_two<M: Copy, const N: usize>(
    nodes: &[Node<M, N>],
    vector: &Vector<N>,
    config: &IndexConfig,
    n: i32,
) -> Vec<QueryResult<M>> {
    let start = std::time::Instant::now();
    let mut index = Index::new(config);
    for node in nodes.iter() {
        index.insert(*node);
    }

    println!("Index two build: {:?}", start.elapsed().as_micros());

    let start = std::time::Instant::now();
    let result = index.query(vector, n);
    println!("Index two query: {:?}", start.elapsed().as_micros());

    result
}

fn search_index_one<M: Copy, const N: usize>(
    nodes: &Vec<Node<M, N>>,
    vector: &Vector<N>,
    config: &IndexConfig,
    n: i32,
) -> Vec<QueryResult<M>> {
    let start = std::time::Instant::now();
    let index = Index::build(nodes, config);
    println!("Index one build: {:?}", start.elapsed().as_micros());

    let start = std::time::Instant::now();
    let result = index.query(vector, n);
    println!("Index one query: {:?}", start.elapsed().as_micros());

    result
}

fn search_exhaustive<M: Copy, const N: usize>(
    nodes: &[Node<M, N>],
    vector: &Vector<N>,
) -> Vec<(&'static str, f32)> {
    let start = std::time::Instant::now();

    let mut result: Vec<(&str, f32)> = nodes
        .iter()
        .map(|node| (node.key, node.vector.euclidean_distance(vector)))
        .collect();
    result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    println!("Exhaustive query: {:?}", start.elapsed().as_micros());
    result
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
