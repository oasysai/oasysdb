use oasysdb::prelude::*;

fn main() {
    // Vector dimension must be uniform.
    let dimension = 128;

    // Replace with your own data.
    let records = Record::many_random(dimension, 100);

    let mut config = Config::default();

    // Optionally set the distance function. Default to Euclidean.
    config.distance = Distance::Cosine;

    // Create a vector collection.
    let collection = Collection::build(&config, &records).unwrap();

    // Optionally save the collection to persist it.
    let mut db = Database::new("data/test").unwrap();
    db.save_collection("vectors", &collection).unwrap();

    // Search for the nearest neighbors.
    let query = Vector::random(dimension);
    let result = collection.search(&query, 5).unwrap();

    for res in result {
        let (id, distance) = (res.id, res.distance);
        println!("{distance:.5} | ID: {id}");
    }
}
