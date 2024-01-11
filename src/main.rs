use oasysdb::index::*;
use oasysdb::vector::Vector;

fn main() {
    let config = IndexConfig::default();

    let keys = vec!["red", "green", "blue"];
    let vectors = vec![
        Vector([255.0, 0.0, 0.0]),
        Vector([0.0, 255.0, 0.0]),
        Vector([0.0, 0.0, 255.0]),
    ];

    let hnsw: IndexGraph<&str, 3, 32> =
        IndexGraph::build(config, &keys, &vectors);

    let light_coral = Vector([240.0, 128.0, 128.0]);
    let nearest = hnsw.search(&light_coral, 2);
    println!("Nearest: {:?}", nearest);
}
